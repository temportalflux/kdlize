use std::marker::PhantomData;
use crate::{error::{MissingChild, MissingEntryValue, MissingTypedEntry, RequiredChild, RequiredValue}, FromKdlNode, FromKdlValue};

pub struct NodeReader<'doc, Context> {
	node: &'doc kdl::KdlNode,
	ctx: &'doc Context,
	entry_cursor: usize,
}

impl<'doc, Context> ToString for NodeReader<'doc, Context> {
	fn to_string(&self) -> String {
		self.node.to_string()
	}
}

impl<'doc, Context> NodeReader<'doc, Context> {
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

	pub fn name(&self) -> &kdl::KdlIdentifier {
		self.node.name()
	}

	pub fn entries(&self) -> &[kdl::KdlEntry] {
		self.node.entries()
	}

	pub fn document(&self) -> Option<&kdl::KdlDocument> {
		self.node.children()
	}
}

impl<'doc, Context> NodeReader<'doc, Context> {
	pub fn peak_opt(&self) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry(self.entry_cursor)
	}
	
	pub fn peak_req(&self) -> Result<&'doc kdl::KdlEntry, MissingEntryValue> {
		self.peak_opt().ok_or_else(|| MissingEntryValue::new_index(self.node.clone(), self.entry_cursor))
	}

	pub fn peak_req_ty(&self) -> Result<&str, MissingTypedEntry> {
		use crate::ext::EntryExt;
		Ok(self.peak_req()?.type_req()?)
	}

	fn increment_cursor(&mut self) -> usize {
		let consumed = self.entry_cursor;
		self.entry_cursor += 1;
		consumed
	}

	fn next_entry(&mut self) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry(self.increment_cursor())
	}

	pub fn next_opt<V: FromKdlValue<'doc>>(&mut self) -> Result<Option<V>, V::Error> {
		let Some(entry) = self.next_entry() else { return Ok(None) };
		Ok(Some(V::from_kdl(entry.value())?))
	}

	pub fn next_req<V: FromKdlValue<'doc>>(&mut self) -> Result<V, RequiredValue<V::Error>> {
		let idx = self.entry_cursor;
		match self.next_opt::<V>() {
			Err(err) => Err(err.into()),
			Ok(Some(value)) => Ok(value),
			Ok(None) => Err(RequiredValue::Missing(MissingEntryValue::new_index(self.node.clone(), idx))),
		}
	}

	fn prop(&self, key: impl AsRef<str>) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry(key.as_ref())
	}

	pub fn prop_opt<V: FromKdlValue<'doc>>(&mut self, key: impl AsRef<str>) -> Result<Option<V>, V::Error> {
		let Some(entry) = self.prop(key) else { return Ok(None) };
		Ok(Some(V::from_kdl(entry.value())?))
	}

	pub fn prop_req<V: FromKdlValue<'doc>>(&mut self, key: impl AsRef<str>) -> Result<V, RequiredValue<V::Error>> {
		match self.prop_opt::<V>(key.as_ref()) {
			Err(err) => Err(err.into()),
			Ok(Some(value)) => Ok(value),
			Ok(None) => Err(RequiredValue::Missing(MissingEntryValue::new_prop(self.node.clone(), key))),
		}
	}

	pub fn iter_children(&'doc self, key: &'static str) -> IterChildNodesWithName<'doc, Context> {
		IterChildNodesWithName(IterChildNodes(self, 0), key)
	}

	pub fn child_opt(&'doc self, key: &'static str) -> Option<Self> {
		self.iter_children(key).next()
	}

	pub fn child_req(&'doc self, key: &'static str) -> Result<Self, MissingChild> {
		self.child_opt(key).ok_or_else(|| MissingChild(self.node.clone(), key))
	}
}
impl<'doc, Context> NodeReader<'doc, Context> {
	pub fn iter_children_t<T>(&'doc self, key: &'static str) -> IterChildNodesWithNameTyped<'doc, Context, T> where T: FromKdlNode<Context> {
		IterChildNodesWithNameTyped::<'doc, Context, T>(self.iter_children(key), PhantomData::<T>)
	}

	pub fn children_t<T>(&'doc self, key: &'static str) -> Result<Vec<T>, T::Error> where T: FromKdlNode<Context> {
		let mut children = Vec::new();
		for child in self.iter_children_t::<T>(key) {
			children.push(child?);
		}
		Ok(children)
	}

	pub fn child_opt_t<T>(&'doc self, key: &'static str) -> Result<Option<T>, T::Error> where T: FromKdlNode<Context> {
		self.iter_children_t::<T>(key).next().transpose()
	}

	pub fn child_req_t<T>(&'doc self, key: &'static str) -> Result<T, RequiredChild<T::Error>> where T: FromKdlNode<Context> {
		let child_opt = self.iter_children_t::<T>(key).next();
		let child_opt = child_opt.transpose()?;
		child_opt.ok_or_else(|| RequiredChild::Missing(MissingChild(self.node.clone(), key)))
	}
}

pub struct IterChildNodes<'doc, Context>(&'doc NodeReader<'doc, Context>, usize);
impl<'doc, Context> Iterator for IterChildNodes<'doc, Context> {
	type Item = NodeReader<'doc, Context>;
	fn next(&mut self) -> Option<Self::Item> {
		let document = self.0.document()?;
		let node = document.nodes().get(self.1)?;
		self.1 += 1;
		Some(NodeReader::<'doc, Context>::new(node, &self.0.ctx))
	}
}

pub struct IterChildNodesWithName<'doc, Context>(IterChildNodes<'doc, Context>, &'static str);
impl<'doc, Context> Iterator for IterChildNodesWithName<'doc, Context> {
	type Item = NodeReader<'doc, Context>;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(reader) = self.0.next() {
			if reader.node.name().value() == self.1 {
				return Some(reader);
			}
		}
		None
	}
}

pub struct IterChildNodesWithNameTyped<'doc, Context, T: FromKdlNode<Context>>(IterChildNodesWithName<'doc, Context>, PhantomData<T>);
impl<'doc, Context, T> Iterator for IterChildNodesWithNameTyped<'doc, Context, T> where T: FromKdlNode<Context> {
	type Item = Result<T, T::Error>;
	fn next(&mut self) -> Option<Self::Item> {
		Some(T::from_kdl(&mut self.0.next()?))
	}
}

#[cfg(test)]
mod test {
  use super::*;

	#[test]
	fn next() -> Result<(), anyhow::Error> {
		let node = kdl::KdlNode::parse("node value1 2 \"value 3\" key=#true")?;
		let mut reader = NodeReader::new(&node, &());
		assert_eq!(reader.next_req::<&str>()?, "value1");
		assert_eq!(reader.next_req::<u32>()?, 2u32);
		assert_eq!(reader.next_req::<&str>()?, "value 3");
		Ok(())
	}

	#[test]
	fn get() -> Result<(), anyhow::Error> {
		let node = kdl::KdlNode::parse("node value1 2 \"value 3\" key=#true")?;
		let mut reader = NodeReader::new(&node, &());
		assert_eq!(reader.prop_req::<bool>("key")?, true);
		Ok(())
	}

	#[test]
	fn children() -> Result<(), anyhow::Error> {
		let node = kdl::KdlNode::parse("root {\nc1 \"yes 1\"\nc2 \"no\"\nc1 \"yes 2\"}")?;
		let reader = NodeReader::new(&node, &());
		let child_nodes = reader.iter_children("c1").map(|reader| reader.to_string()).collect::<Vec<_>>();
		assert_eq!(child_nodes, vec![
			"c1 \"yes 1\"\n".to_owned(),
			"c1 \"yes 2\"".to_owned(),
		]);
		let child_nodes = reader.iter_children("c2").map(|reader| reader.to_string()).collect::<Vec<_>>();
		assert_eq!(child_nodes, vec![
			"c2 \"no\"\n".to_owned(),
		]);
		Ok(())
	}
}
