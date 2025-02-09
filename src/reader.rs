
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
	entry_cursor: usize,
}

impl<'doc, Context> Clone for Node<'doc, Context> {
	fn clone(&self) -> Self {
		Self { node: self.node, ctx: self.ctx, entry_cursor: self.entry_cursor.clone() }
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
			entry_cursor: 0,
		}
	}

	pub fn context(&self) -> &Context {
		&self.ctx
	}

	pub fn name(&self) -> &'doc kdl::KdlIdentifier {
		self.node.name()
	}

	pub fn entries(&self) -> &'doc [kdl::KdlEntry] {
		self.node.entries()
	}

	pub fn document(&self) -> Option<&'doc kdl::KdlDocument> {
		self.node.children()
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

	pub fn iter_children(&self) -> IterChildNodes<'_, 'doc, Context> {
		IterChildNodes(self, 0)
	}

	pub fn children(&self, name: impl Into<kdl::KdlIdentifier>) -> IterChildNodesWithName<'_, 'doc, Context> {
		IterChildNodesWithName(IterChildNodes(self, 0), name.into())
	}

	pub fn child(&self, key: impl Into<kdl::KdlIdentifier>) -> Result<Self, crate::error::MissingChild> {
		let key = key.into();
		let child = self.children(key.clone()).next();
		child.ok_or_else(|| crate::error::MissingChild(self.node.clone(), key))
	}

	pub fn to<T: crate::FromKdlNode<Context>>(&mut self) -> Result<T, T::Error> {
		T::from_kdl(self)
	}
}

pub struct IterChildNodes<'reader, 'doc, Context>(&'reader Node<'doc, Context>, usize);
impl<'reader, 'doc, Context> Iterator for IterChildNodes<'reader, 'doc, Context> {
	type Item = Node<'doc, Context>;
	fn next(&mut self) -> Option<Self::Item> {
		let document = self.0.document()?;
		let node = document.nodes().get(self.1)?;
		self.1 += 1;
		Some(Node::<'doc, Context>::new(node, &self.0.ctx))
	}
}

pub struct IterChildNodesWithName<'reader, 'doc, Context>(IterChildNodes<'reader, 'doc, Context>, kdl::KdlIdentifier);
impl<'reader, 'doc, Context> Iterator for IterChildNodesWithName<'reader, 'doc, Context> {
	type Item = Node<'doc, Context>;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(reader) = self.0.next() {
			if reader.node.name() == &self.1 {
				return Some(reader);
			}
		}
		None
	}
}
impl<'reader, 'doc, Context> IterChildNodesWithName<'reader, 'doc, Context> {
	pub fn to<T: crate::FromKdlNode<Context>>(self) -> IterChildNodesWithNameTyped<'reader, 'doc, Context, T> {
		IterChildNodesWithNameTyped(self, std::marker::PhantomData::default())
	}
}

pub struct IterChildNodesWithNameTyped<'reader, 'doc, Context, T: crate::FromKdlNode<Context>>(
	IterChildNodesWithName<'reader, 'doc, Context>,
	std::marker::PhantomData<T>,
);
impl<'reader, 'doc, Context, T> Iterator for IterChildNodesWithNameTyped<'reader, 'doc, Context, T>
where
	T: crate::FromKdlNode<Context>,
{
	type Item = Result<T, T::Error>;
	fn next(&mut self) -> Option<Self::Item> {
		Some(T::from_kdl(&mut self.0.next()?))
	}
}

pub trait EntryExt {
	fn typed(&self) -> Result<&str, crate::error::MissingEntryType>;
	fn to<T>(&self) -> Result<T, T::Error> where T: crate::FromKdlValue;
}
impl EntryExt for kdl::KdlEntry {
	fn typed(&self) -> Result<&str, crate::error::MissingEntryType> {
		let ty = self.ty().map(kdl::KdlIdentifier::value);
		ty.ok_or_else(|| crate::error::MissingEntryType(self.clone()))
	}

	fn to<T>(&self) -> Result<T, T::Error> where T: crate::FromKdlValue {
		T::from_kdl(self.value())
	}
}

#[cfg(test)]
mod test {
	use crate::error::QueryError;
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
	fn peak_value() -> Result<(), QueryError> {
		let node = node();
		let reader = Node::new(&node, &());
		assert_eq!(reader.peak()?.to::<u32>()?, 42);
		assert_eq!(reader.entry_cursor, 0);
		Ok(())
	}

	#[test]
	fn next_typed() -> Result<(), QueryError> {
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
	fn next_value() -> Result<(), QueryError> {
		let node = node();
		let mut reader = Node::new(&node, &());
		assert_eq!(reader.next()?.to::<u32>()?, 42);
		assert_eq!(reader.next()?.to::<bool>()?, false);
		assert_eq!(reader.next()?.to::<String>()?, "hello");
		Ok(())
	}

	#[test]
	fn property_value() -> Result<(), QueryError> {
		let node = node();
		let reader = Node::new(&node, &());
		assert_eq!(reader.prop("some_key")?.to::<f32>()?, 3.0);
		Ok(())
	}

	#[test]
	fn child_node() -> Result<(), QueryError> {
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
	fn child_next_value() -> Result<(), QueryError> {
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
	fn child_fromnode() -> Result<(), QueryError> {
		#[derive(PartialEq, Debug)]
		struct ExampleData {
			value: u32,
		}
		impl crate::FromKdlNode<()> for ExampleData {
			type Error = QueryError;
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
	fn child_all_fromnode() -> Result<(), QueryError> {
		#[derive(PartialEq, Debug)]
		struct ExampleData {
			string: String,
			flag: bool,
		}
		impl crate::FromKdlNode<()> for ExampleData {
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
