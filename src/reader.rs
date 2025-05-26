/*
	I wanted a different syntax such as:

	let mut reader = Node::new(&kdl_node, &context)?;
	let value: u32 = reader.at(Next > u32)?;
	let value: &str = reader.at(Next > str)?;
	let value: Option<String> = reader.at(Next > String)?.ok();
	let (type, kdl_value) = reader.at(Peak > Entry)?;
	let child: reader::Node = reader.at(Child("c1"))?;
	let value: f32 = reader.at(Child("c1") > Property("key") > f32)?;
	let values: Vec<String> = reader.at(Children("c2") > Next > String)?.collect();

	impl FromKdlNode for T {}
	let value: Option<T> = reader.at(Child("c4") > T)?.ok();
	let values: Vec<T> = reader.at(Children("c3") > T)?.collect();

	but reader::Node contains `&'doc KdlNode`, and overriding trait associated types did not play well with that lifetime.

	So I ended up with the more verbose exemplified in test module below.
*/

pub struct Node<'doc, Context> {
	node: &'doc kdl::KdlNode,
	ctx: &'doc Context,
	is_child: bool,
	entry_cursor: usize,
}

impl<'doc, Context> Clone for Node<'doc, Context> {
	fn clone(&self) -> Self {
		Self {
			node: self.node,
			ctx: self.ctx,
			is_child: self.is_child,
			entry_cursor: self.entry_cursor.clone(),
		}
	}
}

impl<'doc, Context> ToString for Node<'doc, Context> {
	fn to_string(&self) -> String {
		self.node.to_string()
	}
}

impl<'doc, Context> Node<'doc, Context> {
	pub fn new(node: &'doc kdl::KdlNode, ctx: &'doc Context) -> Self {
		Self {
			node,
			ctx,
			is_child: false,
			entry_cursor: 0,
		}
	}

	pub fn context(&self) -> &Context {
		&self.ctx
	}

	pub fn is_child(&self) -> bool {
		self.is_child
	}

	pub fn name(&self) -> &'doc kdl::KdlIdentifier {
		self.node.name()
	}

	pub fn entries(&self) -> &'doc [kdl::KdlEntry] {
		self.node.entries()
	}

	pub fn document(&self) -> Result<&'doc kdl::KdlDocument, crate::error::MissingNodeDocument> {
		match self.node.children() {
			Some(doc) => Ok(doc),
			None => {
				let src = self.node.to_string();
				Err(crate::error::MissingNodeDocument {
					node_span: (0, src.len()).into(),
					src,
				})
			}
		}
	}

	pub fn has_children(&self) -> bool {
		let Ok(doc) = self.document() else { return false };
		!doc.nodes().is_empty()
	}
}

impl<'doc, Context> Node<'doc, Context> {
	pub fn peak(&self) -> Result<&'doc kdl::KdlEntry, crate::error::MissingEntry> {
		let entry = self.node.entry(self.entry_cursor);
		entry.ok_or_else(|| crate::error::MissingEntry::new_index(self.node.clone(), self.entry_cursor))
	}

	pub fn next(&mut self) -> Result<&'doc kdl::KdlEntry, crate::error::MissingEntry> {
		let entry = self.node.entry(self.entry_cursor);
		if entry.is_some() {
			self.entry_cursor += 1;
		}
		entry.ok_or_else(|| crate::error::MissingEntry::new_index(self.node.clone(), self.entry_cursor))
	}

	pub fn prop(&self, key: impl AsRef<str>) -> Result<&'doc kdl::KdlEntry, crate::error::MissingEntry> {
		let entry = self.node.entry(key.as_ref());
		entry.ok_or_else(|| crate::error::MissingEntry::new_prop(self.node.clone(), key))
	}

	pub fn iter_children(&self) -> IterChildNodes<IterDocumentNodes<'doc>, &'doc Context> {
		let iter_doc = self.document().ok().map(|doc| IterDocumentNodes(doc, 0));
		IterChildNodes(iter_doc, &self.ctx)
	}

	pub fn children(
		&self,
		name: impl Into<kdl::KdlIdentifier>,
	) -> IterChildNodes<IterDocumentNodesWithName<'doc>, &'doc Context> {
		let iter_doc = self
			.document()
			.ok()
			.map(|doc| IterDocumentNodesWithName(IterDocumentNodes(doc, 0), name.into()));
		IterChildNodes(iter_doc, &self.ctx)
	}

	pub fn child(&self, key: impl Into<kdl::KdlIdentifier>) -> Result<Self, crate::error::NodeMissingChild> {
		let key = key.into();
		let child = self.children(key.clone()).next();
		child.ok_or_else(|| {
			let src = self.node.to_string();
			let label = format!("missing child named {:?}", key.value());
			let span = miette::LabeledSpan::new_primary_with_span(Some(label), (0, src.len()));
			crate::error::NodeMissingChild {
				src,
				span,
				child_name: key, 
			}
		})
	}

	pub fn to<T: crate::FromKdlNode<'doc, Context>>(&mut self) -> Result<T, T::Error> {
		T::from_kdl(self)
	}
}

pub struct IterDocumentNodes<'doc>(&'doc kdl::KdlDocument, usize);
impl<'doc> IterDocumentNodes<'doc> {
	pub fn opt(doc: Option<&'doc kdl::KdlDocument>) -> Option<Self> {
		doc.map(|doc| Self(doc, 0))
	}
}
impl<'doc> Iterator for IterDocumentNodes<'doc> {
	type Item = &'doc kdl::KdlNode;
	fn next(&mut self) -> Option<Self::Item> {
		let node = self.0.nodes().get(self.1)?;
		self.1 += 1;
		Some(node)
	}
}

pub struct IterDocumentNodesWithName<'doc>(IterDocumentNodes<'doc>, kdl::KdlIdentifier);
impl<'doc> Iterator for IterDocumentNodesWithName<'doc> {
	type Item = &'doc kdl::KdlNode;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(node) = self.0.next() {
			if node.name().value() == self.1.value() {
				return Some(node);
			}
		}
		None
	}
}

pub struct IterChildNodes<Iter, Context>(Option<Iter>, Context);
impl<'doc, Context: 'doc> Iterator for IterChildNodes<IterDocumentNodes<'doc>, &'doc Context> {
	type Item = Node<'doc, Context>;
	fn next(&mut self) -> Option<Self::Item> {
		let iter_doc = self.0.as_mut()?;
		let node = iter_doc.next()?;
		Some(Node {
			node,
			ctx: self.1,
			is_child: true,
			entry_cursor: 0,
		})
	}
}
impl<'doc, Context: 'doc> Iterator for IterChildNodes<IterDocumentNodesWithName<'doc>, &'doc Context> {
	type Item = Node<'doc, Context>;
	fn next(&mut self) -> Option<Self::Item> {
		let iter_doc = self.0.as_mut()?;
		let node = iter_doc.next()?;
		Some(Node {
			node,
			ctx: self.1,
			is_child: true,
			entry_cursor: 0,
		})
	}
}

impl<'doc, Context: 'doc, Iter> IterChildNodes<Iter, &'doc Context>
where
	Iter: Iterator,
{
	pub fn value(self) -> IterNodeFirstValue<Self> {
		IterNodeFirstValue(self)
	}

	pub fn prop<S: AsRef<str>>(self, key: S) -> IterNodePropValue<Self, S> {
		IterNodePropValue(self, key)
	}

	pub fn to<T: crate::FromKdlNode<'doc, Context>>(self) -> IterNodeTyped<Self, T> {
		IterNodeTyped(self, std::marker::PhantomData::default())
	}
}

pub struct IterNodeFirstValue<Iter>(Iter);
impl<'doc, Context: 'doc, Iter> Iterator for IterNodeFirstValue<Iter>
where
	Iter: Iterator<Item = Node<'doc, Context>>,
{
	type Item = Result<&'doc kdl::KdlEntry, crate::error::MissingEntry>;
	fn next(&mut self) -> Option<Self::Item> {
		let mut node = self.0.next()?;
		Some(node.next())
	}
}
impl<'doc, Context: 'doc, Iter> IterNodeFirstValue<Iter>
where
	Iter: Iterator<Item = Node<'doc, Context>>,
{
	pub fn to<T: crate::FromKdlValue<'doc>>(self) -> IterNodeValueTyped<Self, T> {
		IterNodeValueTyped(self, std::marker::PhantomData::default())
	}
}

pub struct IterNodePropValue<Iter, S: AsRef<str>>(Iter, S);
impl<'doc, Context: 'doc, Iter, S: AsRef<str>> Iterator for IterNodePropValue<Iter, S>
where
	Iter: Iterator<Item = Node<'doc, Context>>,
{
	type Item = Result<&'doc kdl::KdlEntry, crate::error::MissingEntry>;
	fn next(&mut self) -> Option<Self::Item> {
		let node = self.0.next()?;
		Some(node.prop(self.1.as_ref()))
	}
}
impl<'doc, Context: 'doc, Iter, S: AsRef<str>> IterNodePropValue<Iter, S>
where
	Iter: Iterator<Item = Node<'doc, Context>>,
{
	pub fn to<T: crate::FromKdlValue<'doc>>(self) -> IterNodeValueTyped<Self, T> {
		IterNodeValueTyped(self, std::marker::PhantomData::default())
	}
}

pub struct IterNodeValueTyped<Iter, T>(Iter, std::marker::PhantomData<T>);
impl<'doc, Iter, T> Iterator for IterNodeValueTyped<Iter, T>
where
	Iter: Iterator<Item = Result<&'doc kdl::KdlEntry, crate::error::MissingEntry>>,
	T: crate::FromKdlValue<'doc>,
	miette::Report: From<T::Error>,
{
	type Item = Result<Result<T, FailedToParseValue>, crate::error::MissingEntry>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.0.next()? {
			Err(missing_entry) => Some(Err(missing_entry)),
			Ok(entry) => Some(Ok(entry.to::<T>())),
		}
	}
}
impl<'doc, Iter, T: crate::FromKdlValue<'doc>> IterNodeValueTyped<Iter, T>
where
	Self: Iterator<Item = Result<Result<T, FailedToParseValue>, crate::error::MissingEntry>>,
{
	pub fn collect<C: FromIterator<T>>(self) -> Result<Result<C, FailedToParseValue>, crate::error::MissingEntry> {
		match <Self as Iterator>::collect::<Result<Vec<_>, _>>(self) {
			Ok(inner) => match inner.into_iter().collect::<Result<C, _>>() {
				Ok(values) => Ok(Ok(values)),
				Err(parse_err) => Ok(Err(parse_err)),
			},
			Err(missing) => Err(missing),
		}
	}
}

pub struct IterNodeTyped<Iter, T>(Iter, std::marker::PhantomData<T>);
impl<'doc, Context: 'doc, Iter, T> Iterator for IterNodeTyped<Iter, T>
where
	Iter: Iterator<Item = Node<'doc, Context>>,
	T: crate::FromKdlNode<'doc, Context>,
{
	type Item = Result<T, T::Error>;
	fn next(&mut self) -> Option<Self::Item> {
		Some(T::from_kdl(&mut self.0.next()?))
	}
}
impl<'doc, Context: 'doc, Iter, T> IterNodeTyped<Iter, T>
where
	Iter: Iterator<Item = Node<'doc, Context>>,
	T: crate::FromKdlNode<'doc, Context>,
{
	pub fn collect<C: FromIterator<T>>(self) -> Result<C, T::Error> {
		<Self as Iterator>::collect(self)
	}
}

pub trait EntryExt<'doc> {
	fn typed(&'doc self) -> Result<&'doc str, crate::error::MissingEntryType>;
	fn to<T>(&'doc self) -> Result<T, FailedToParseValue>
	where
		T: crate::FromKdlValue<'doc>, miette::Report: From<T::Error>;
}
impl<'doc> EntryExt<'doc> for kdl::KdlEntry {
	fn typed(&'doc self) -> Result<&'doc str, crate::error::MissingEntryType> {
		match self.ty().map(kdl::KdlIdentifier::value) {
			Some(value) => Ok(value),
			None => {
				// TODO: Would be nice if the source was the node itself
				let src = self.to_string();
				Err(crate::error::MissingEntryType {
					span: (0, src.len()).into(),
					src,
					value: self.clone(),
				})
			}
		}
	}

	fn to<T>(&'doc self) -> Result<T, FailedToParseValue>
	where
		T: crate::FromKdlValue<'doc>, miette::Report: From<T::Error>,
	{
		let result = T::from_kdl(self.value());
		let parsed_value = result.map_err(|err| {
			let span = self.span();
			FailedToParseValue { span, err: miette::Report::from(err) }
		})?;
		Ok(parsed_value)
	}
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse value: {err:?}")]
pub struct FailedToParseValue {
	span: miette::SourceSpan,
	err: miette::Report,
}
impl miette::Diagnostic for FailedToParseValue {
	fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
		Some(Box::new("kdlize::failed_to_parse_value"))
	}

	fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
		Some(Box::new(vec![
			miette::LabeledSpan::new_with_span(Some(format!("{}", self.err)), self.span.clone()),
		].into_iter()))
	}
}

pub trait EntryOptExt<'doc> {
	fn to<T>(&self) -> Result<Option<T>, FailedToParseValue>
	where
		T: crate::FromKdlValue<'doc>, miette::Report: From<T::Error>; // T::Error: miette::Diagnostic + Send + Sync + 'static;
}
impl<'doc> EntryOptExt<'doc> for Option<&'doc kdl::KdlEntry> {
	fn to<T>(&self) -> Result<Option<T>, FailedToParseValue>
	where
		T: crate::FromKdlValue<'doc>, miette::Report: From<T::Error>, // T::Error: miette::Diagnostic + Send + Sync + 'static,
	{
		let Some(entry) = self else { return Ok(None) };
		let result = T::from_kdl(entry.value());
		let parsed_value = result.map_err(|err| {
			let span = entry.span();
			FailedToParseValue { span, err: miette::Report::from(err) }
		})?;
		Ok(Some(parsed_value))
	}
}

pub trait NodeOptExt<'doc> {
	type Context;
	fn next(self) -> Result<Option<&'doc kdl::KdlEntry>, crate::error::MissingEntry>;
	fn prop(self, key: impl AsRef<str>) -> Result<Option<&'doc kdl::KdlEntry>, crate::error::MissingEntry>;
	fn to<T>(self) -> Result<Option<T>, T::Error>
	where
		T: crate::FromKdlNode<'doc, Self::Context>;
}
impl<'doc, Context> NodeOptExt<'doc> for Option<Node<'doc, Context>> {
	type Context = Context;

	fn next(self) -> Result<Option<&'doc kdl::KdlEntry>, crate::error::MissingEntry> {
		let Some(mut node) = self else { return Ok(None) };
		node.next().map(|v| Some(v))
	}

	fn prop(self, key: impl AsRef<str>) -> Result<Option<&'doc kdl::KdlEntry>, crate::error::MissingEntry> {
		let Some(node) = self else { return Ok(None) };
		node.prop(key).map(|v| Some(v))
	}

	fn to<T>(self) -> Result<Option<T>, T::Error>
	where
		T: crate::FromKdlNode<'doc, Self::Context>,
	{
		let Some(mut node) = self else { return Ok(None) };
		Ok(Some(T::from_kdl(&mut node)?))
	}
}

pub trait DocumentExt {
	fn iter_children<'doc>(&'doc self) -> IterDocumentNodes<'doc>;
	fn children<'doc>(&'doc self, name: impl Into<kdl::KdlIdentifier>) -> IterDocumentNodesWithName<'doc>;
	fn child<'doc>(&'doc self, key: impl Into<kdl::KdlIdentifier>) -> Option<&'doc kdl::KdlNode>;
}
impl DocumentExt for kdl::KdlDocument {
	fn iter_children<'doc>(&'doc self) -> IterDocumentNodes<'doc> {
		IterDocumentNodes(self, 0)
	}

	fn children<'doc>(&'doc self, name: impl Into<kdl::KdlIdentifier>) -> IterDocumentNodesWithName<'doc> {
		IterDocumentNodesWithName(IterDocumentNodes(self, 0), name.into())
	}

	fn child<'doc>(&'doc self, key: impl Into<kdl::KdlIdentifier>) -> Option<&'doc kdl::KdlNode> {
		let key = key.into();
		self.children(key.clone()).next()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	fn empty() -> kdl::KdlNode {
		kdl::KdlNode::new("empty")
	}

	fn node() -> kdl::KdlNode {
		let mut node = kdl::KdlNode::new("node");
		node.entries_mut().push(kdl::KdlEntry::new(42));
		node.entries_mut().push({
			let mut entry = kdl::KdlEntry::new(false);
			entry.set_ty("FlagName");
			entry
		});
		node.entries_mut().push(kdl::KdlEntry::new("hello"));
		node.entries_mut().push(kdl::KdlEntry::new_prop("some_key", 3.0));
		node.set_children({
			let mut doc = kdl::KdlDocument::new();
			doc.nodes_mut().push({
				let mut node = kdl::KdlNode::new("child1");
				node.entries_mut().push(kdl::KdlEntry::new(42));
				node.entries_mut().push({
					let mut entry = kdl::KdlEntry::new("StrValue");
					entry.set_ty("CustomType");
					entry
				});
				node
			});
			doc.nodes_mut().push(empty());
			doc.nodes_mut().push({
				let mut node = kdl::KdlNode::new("child2");
				node.entries_mut().push(kdl::KdlEntry::new("ValueA"));
				node.entries_mut().push(kdl::KdlEntry::new_prop("flag", true));
				node
			});
			doc.nodes_mut().push({
				let mut node = kdl::KdlNode::new("child2");
				node.entries_mut().push(kdl::KdlEntry::new("ValueB"));
				node.entries_mut().push(kdl::KdlEntry::new_prop("flag", false));
				node
			});
			doc
		});
		node
	}

	#[test]
	fn peak_value() -> Result<(), miette::Error> {
		let node = node();
		let reader = Node::new(&node, &());
		assert_eq!(reader.peak()?.to::<u32>()?, 42);
		assert_eq!(reader.entry_cursor, 0);
		Ok(())
	}

	#[test]
	fn next_typed() -> Result<(), miette::Error> {
		let node = node();
		let mut reader = Node::new(&node, &());
		let _ = reader.next();
		assert_eq!(
			(reader.peak()?.typed()?, reader.peak()?.value()),
			("FlagName", &kdl::KdlValue::Bool(false))
		);
		assert_eq!(reader.entry_cursor, 1);
		Ok(())
	}

	#[test]
	fn next_value() -> Result<(), miette::Error> {
		let node = node();
		let mut reader = Node::new(&node, &());
		assert_eq!(reader.next()?.to::<u32>()?, 42);
		assert_eq!(reader.next()?.to::<bool>()?, false);
		assert_eq!(reader.next()?.to::<String>()?, "hello");
		Ok(())
	}

	#[test]
	fn property_value() -> Result<(), miette::Error> {
		let node = node();
		let reader = Node::new(&node, &());
		assert_eq!(reader.prop("some_key")?.to::<f32>()?, 3.0);
		Ok(())
	}

	#[test]
	fn child_node() -> Result<(), miette::Error> {
		let node = node();
		let reader = Node::new(&node, &());
		let value = reader.child("child1")?;
		let expected = {
			let mut node = kdl::KdlNode::new("child1");
			node.entries_mut().push(kdl::KdlEntry::new(42));
			node.entries_mut().push({
				let mut entry = kdl::KdlEntry::new("StrValue");
				entry.set_ty("CustomType");
				entry
			});
			node
		};
		assert_eq!(value.node, &expected);
		Ok(())
	}

	#[test]
	fn child_next_value() -> Result<(), miette::Error> {
		let node = node();
		let reader = Node::new(&node, &());
		let value = reader.child("child1")?.next()?.to::<u32>()?;
		assert_eq!(value, 42);
		Ok(())
	}

	#[test]
	fn child_all() {
		let node = node();
		let reader = Node::new(&node, &());
		let mut iter = reader.children("child2");
		assert_eq!(
			iter.next().map(|reader| reader.node),
			Some(&{
				let mut node = kdl::KdlNode::new("child2");
				node.entries_mut().push(kdl::KdlEntry::new("ValueA"));
				node.entries_mut().push(kdl::KdlEntry::new_prop("flag", true));
				node
			})
		);
		assert_eq!(
			iter.next().map(|reader| reader.node),
			Some(&{
				let mut node = kdl::KdlNode::new("child2");
				node.entries_mut().push(kdl::KdlEntry::new("ValueB"));
				node.entries_mut().push(kdl::KdlEntry::new_prop("flag", false));
				node
			})
		);
		assert_eq!(iter.next().map(|reader| reader.node), None);
	}

	#[test]
	fn child_fromnode() -> Result<(), miette::Error> {
		#[derive(PartialEq, Debug)]
		struct ExampleData {
			value: u32,
		}
		impl<'doc> crate::FromKdlNode<'doc, ()> for ExampleData {
			type Error = miette::Error;
			fn from_kdl(node: &mut super::Node<()>) -> Result<Self, Self::Error> {
				let value = node.next()?.to::<u32>()?;
				Ok(Self { value })
			}
		}
		let node = node();
		let reader = Node::new(&node, &());
		let value = reader.child("child1")?.to::<ExampleData>()?;
		assert_eq!(value, ExampleData { value: 42 });
		Ok(())
	}

	#[test]
	fn child_all_fromnode() -> Result<(), miette::Error> {
		#[derive(PartialEq, Debug)]
		struct ExampleData {
			string: String,
			flag: bool,
		}
		impl<'doc> crate::FromKdlNode<'doc, ()> for ExampleData {
			type Error = crate::error::QueryError;
			fn from_kdl(node: &mut super::Node<()>) -> Result<Self, Self::Error> {
				let string = node.next()?.to::<String>()?;
				let flag = node.prop("flag")?.to::<bool>()?;
				Ok(Self { string, flag })
			}
		}
		let node = node();
		let reader = Node::new(&node, &());
		let mut iter = reader.children("child2").to::<ExampleData>();
		assert_eq!(
			iter.next(),
			Some(Ok(ExampleData {
				string: "ValueA".into(),
				flag: true
			}))
		);
		assert_eq!(
			iter.next(),
			Some(Ok(ExampleData {
				string: "ValueB".into(),
				flag: false
			}))
		);
		assert_eq!(iter.next(), None);
		Ok(())
	}
}
