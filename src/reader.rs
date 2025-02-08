use crate::{
	error::{MissingChild, MissingEntryType, MissingEntryValue},
	FromKdlNode, FromKdlValue,
};
use std::marker::PhantomData;

pub struct Node<'doc, Context> {
	node: &'doc kdl::KdlNode,
	ctx: &'doc Context,
	entry_cursor: usize,
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

pub struct IterChildNodesWithNameTyped<'reader, 'doc, Context, T: FromKdlNode<Context>>(
	IterChildNodesWithName<'reader, 'doc, Context>,
	PhantomData<T>,
);
impl<'reader, 'doc, Context, T> Iterator for IterChildNodesWithNameTyped<'reader, 'doc, Context, T>
where
	T: FromKdlNode<Context>,
{
	type Item = Result<T, T::Error>;
	fn next(&mut self) -> Option<Self::Item> {
		Some(T::from_kdl(&mut self.0.next()?))
	}
}

pub struct Optional<T>(pub T);
impl<'reader, 'node, Context> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Peak<Value>>
{
	type Output = Option<<Value as ReadKdl<&'node kdl::KdlEntry>>::Output>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.0.read(input).ok()
	}
}
impl<'reader, 'node, Context> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Next<Value>>
{
	type Output = Option<<Value as ReadKdl<&'node kdl::KdlEntry>>::Output>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.0.read(input).ok()
	}
}
impl<'reader, 'node, Context, K: AsRef<str>> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Property<K, Value>>
{
	type Output = Option<<Value as ReadKdl<&'node kdl::KdlEntry>>::Output>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.0.read(input).ok()
	}
}
impl<'reader, 'node, Context> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Peak<Typed<Value>>>
{
	type Output = Option<<Typed<Value> as ReadKdl<&'node kdl::KdlEntry>>::Output>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.0.read(input).ok()
	}
}
impl<'reader, 'node, Context> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Next<Typed<Value>>>
{
	type Output = Option<<Typed<Value> as ReadKdl<&'node kdl::KdlEntry>>::Output>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.0.read(input).ok()
	}
}
impl<'reader, 'node, Context, K: AsRef<str>> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Property<K, Typed<Value>>>
{
	type Output = Option<<Typed<Value> as ReadKdl<&'node kdl::KdlEntry>>::Output>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.0.read(input).ok()
	}
}
impl<'reader, 'node, Context, V> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Peak<FromValue<V>>> where V: FromKdlValue
{
	type Output = Result<Option<V>, V::Error>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.0.read(input) {
			Ok(value) => Ok(Some(value)),
			Err(crate::error::RequiredValue::Missing(_)) => Ok(None),
			Err(crate::error::RequiredValue::Parse(v_error)) => Err(v_error),
		}
	}
}
impl<'reader, 'node, Context, V> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Next<FromValue<V>>> where V: FromKdlValue
{
	type Output = Result<Option<V>, V::Error>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.0.read(input) {
			Ok(value) => Ok(Some(value)),
			Err(crate::error::RequiredValue::Missing(_)) => Ok(None),
			Err(crate::error::RequiredValue::Parse(v_error)) => Err(v_error),
		}
	}
}
impl<'reader, 'node, Context, V, K: AsRef<str>> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Property<K, FromValue<V>>> where V: FromKdlValue
{
	type Output = Result<Option<V>, V::Error>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.0.read(input) {
			Ok(value) => Ok(Some(value)),
			Err(crate::error::RequiredValue::Missing(_)) => Ok(None),
			Err(crate::error::RequiredValue::Parse(v_error)) => Err(v_error),
		}
	}
}
impl<'reader, 'node, Context, V> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Peak<Typed<FromValue<V>>>> where V: FromKdlValue
{
	type Output = Result<Option<(Result<&'node str, MissingEntryType>, V)>, V::Error>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.0.read(input) {
			Ok((ty, value)) => Ok(Some((ty, value))),
			Err(crate::error::RequiredValue::Missing(_)) => Ok(None),
			Err(crate::error::RequiredValue::Parse(v_error)) => Err(v_error),
		}
	}
}
impl<'reader, 'node, Context, V> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Next<Typed<FromValue<V>>>> where V: FromKdlValue
{
	type Output = Result<Option<(Result<&'node str, MissingEntryType>, V)>, V::Error>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.0.read(input) {
			Ok((ty, value)) => Ok(Some((ty, value))),
			Err(crate::error::RequiredValue::Missing(_)) => Ok(None),
			Err(crate::error::RequiredValue::Parse(v_error)) => Err(v_error),
		}
	}
}
impl<'reader, 'node, Context, V, K: AsRef<str>> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Property<K, Typed<FromValue<V>>>> where V: FromKdlValue
{
	type Output = Result<Option<(Result<&'node str, MissingEntryType>, V)>, V::Error>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.0.read(input) {
			Ok((ty, value)) => Ok(Some((ty, value))),
			Err(crate::error::RequiredValue::Missing(_)) => Ok(None),
			Err(crate::error::RequiredValue::Parse(v_error)) => Err(v_error),
		}
	}
}

impl<'reader, 'node, Context, V, K: Into<kdl::KdlIdentifier>> ReadKdl<&'reader mut Node<'node, Context>> for Optional<Child<K, Next<FromValue<V>>>> where V: FromKdlValue
{
	type Output = Result<Option<V>, crate::error::RequiredValue<V::Error>>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.0.read(input) {
			Ok(Ok(value)) => Ok(Some(value)),
			Ok(Err(required_error)) => Err(required_error),
			Err(_missing_child) => Ok(None),
		}
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
	fn optional_result_parsing() {
		let node = node();
		let mut reader = Node::new(&node, &());

		let _value = reader.at(Optional(Peak(Value)));
		let _value = reader.at(Optional(Next(Value)));
		let _value = reader.at(Optional(Property("key", Value)));
		let _value = reader.at(Optional(Peak(Typed(Value))));
		let _value = reader.at(Optional(Next(Typed(Value))));
		let _value = reader.at(Optional(Property("key", Typed(Value))));
		let _value = reader.at(Optional(Peak(FromValue::<u32>::new())));
		let _value = reader.at(Optional(Next(FromValue::<u32>::new())));
		let _value = reader.at(Optional(Property("key", FromValue::<u32>::new())));
		let _value = reader.at(Optional(Peak(Typed(FromValue::<u32>::new()))));
		let _value = reader.at(Optional(Next(Typed(FromValue::<u32>::new()))));
		let _value = reader.at(Optional(Property("key", Typed(FromValue::<u32>::new()))));

		let _value = reader.at(Optional(Child("value", Next(FromValue::<u32>::new()))));

	}

	#[test]
	fn peak_value() {
		let node = node();
		let mut reader = Node::new(&node, &());
		assert_eq!(reader.at(Peak(FromValue::<u32>::new())), Ok(42));
		assert_eq!(reader.entry_cursor, 0);
	}

	#[test]
	fn next_typed() {
		let node = node();
		let mut reader = Node::new(&node, &());
		let _ = reader.at(Next(Value));
		assert_eq!(
			reader.at(Peak(Typed(Value))),
			Ok((Ok("FlagName"), &kdl::KdlValue::Bool(false)))
		);
		assert_eq!(reader.entry_cursor, 1);
	}

	#[test]
	fn next_value() {
		let node = node();
		let mut reader = Node::new(&node, &());
		assert_eq!(reader.at(Next(FromValue::<u32>::new())), Ok(42));
		assert_eq!(reader.at(Next(FromValue::<bool>::new())), Ok(false));
		assert_eq!(reader.at(Next(FromValue::<String>::new())), Ok("hello".into()));
	}

	#[test]
	fn property_value() {
		let node = node();
		let mut reader = Node::new(&node, &());
		assert_eq!(reader.at(Property("some_key", FromValue::<f32>::new())), Ok(3.0));
	}

	#[test]
	fn child_node() {
		let node = node();
		let mut reader = Node::new(&node, &());
		let value = reader.at(Child("child1", Reader));
		let value = value.map(|reader| reader.node);
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
		assert_eq!(value, Ok(&expected));
	}

	#[test]
	fn child_next_value() {
		let node = node();
		let mut reader = Node::new(&node, &());
		let value = reader.at(Child("child1", Next(FromValue::<u32>::new())));
		assert_eq!(value, Ok(Ok(42)));
	}

	#[test]
	fn child_all() {
		let node = node();
		let mut reader = Node::new(&node, &());
		let mut iter = reader.at(Children("child2", Reader));
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
	fn child_fromnode() {
		#[derive(PartialEq, Debug)]
		struct ExampleData {
			value: u32,
		}
		impl FromKdlNode<()> for ExampleData {
			type Error = crate::error::QueryError;
			fn from_kdl(node: &mut super::Node<()>) -> Result<Self, Self::Error> {
				let value = node.at(Next(FromValue::<u32>::new()))?;
				Ok(Self { value })
			}
		}
		let node = node();
		let mut reader = Node::new(&node, &());
		let value = reader.at(Child("child1", FromNode::<ExampleData>::new()));
		assert_eq!(value, Ok(Ok(ExampleData { value: 42 })));
	}

	#[test]
	fn child_all_fromnode() {
		#[derive(PartialEq, Debug)]
		struct ExampleData {
			string: String,
			flag: bool,
		}
		impl FromKdlNode<()> for ExampleData {
			type Error = crate::error::QueryError;
			fn from_kdl(node: &mut super::Node<()>) -> Result<Self, Self::Error> {
				let string = node.at(Next(FromValue::<String>::new()))?;
				let flag = node.at(Property("flag", FromValue::<bool>::new()))?;
				Ok(Self { string, flag })
			}
		}
		let node = node();
		let mut reader = Node::new(&node, &());
		let mut iter = reader.at(Children("child2", FromNode::<ExampleData>::new()));
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
	}
}

/*
	let node: reader::Node<'doc, Context>;
	[peak, next, prop, child]
	[req, opt, all]

	node.opt(Peak(Entry)) -> Option<KdlEntry>
	node.opt(Peak(Ty)) -> Option<&str>
	node.req(Peak(Value::<i32>)) -> Result<i32>
	node.req(Next(Entry)) -> Result<KdlEntry>
	node.opt(Next(Value::<&str>)) -> Option<&str>
	node.opt(Property("name", Value::<bool>)) -> Option<bool>
	node.req(Property("name", Ty)) -> Result<&str>
	node.opt(Child("name", Next(Value::<T: FromKdlValue>))) -> Option<T>
	node.req(Child("name", Parsed::<T: FromKdlNode>)) -> Result<T>
	node.all("name", Next(Value::<T: FromKdlValue>)) -> Result<Vec<T>>
	node.all("name", Reader) -> Result<Vec<reader::Node>>
*/

impl<'reader, 'doc, Context> Node<'doc, Context> {
	pub fn at<T>(&'reader mut self, query: T) -> T::Output
	where
		T: ReadKdl<&'reader mut Self>,
	{
		query.read(self)
	}
}

pub trait NodeToEntry<T> {
	fn get_entry<'reader, 'node, Context>(self, input: &'reader mut Node<'node, Context>) -> Result<T::Output, MissingEntryValue> where T: ReadKdl<&'node kdl::KdlEntry>;
}

#[derive(Clone)]
pub struct Peak<T>(pub T);
impl<T> NodeToEntry<T> for Peak<T> {
	fn get_entry<'reader, 'node, Context>(self, input: &'reader mut Node<'node, Context>) -> Result<T::Output, MissingEntryValue> where T: ReadKdl<&'node kdl::KdlEntry> {
		let entry = input.node.entry(input.entry_cursor);
		let entry = entry.ok_or(MissingEntryValue::new_index(input.node.clone(), input.entry_cursor))?;
		Ok(self.0.read(entry))
	}
}

#[derive(Clone)]
pub struct Next<T>(pub T);
impl<T> NodeToEntry<T> for Next<T> {
	fn get_entry<'reader, 'node, Context>(self, input: &'reader mut Node<'node, Context>) -> Result<T::Output, MissingEntryValue> where T: ReadKdl<&'node kdl::KdlEntry> {
		let idx = input.entry_cursor;
		let entry = input.node.entry(idx);
		input.entry_cursor += 1;
		let entry = entry.ok_or(MissingEntryValue::new_index(input.node.clone(), idx))?;
		Ok(self.0.read(entry))
	}
}

#[derive(Clone)]
pub struct Property<K, T>(pub K, pub T);
impl<K: AsRef<str>, T> NodeToEntry<T> for Property<K, T> {
	fn get_entry<'reader, 'node, Context>(self, input: &'reader mut Node<'node, Context>) -> Result<T::Output, MissingEntryValue> where T: ReadKdl<&'node kdl::KdlEntry> {
		let entry = input.node.entry(self.0.as_ref());
		let entry = entry.ok_or(MissingEntryValue::new_prop(input.node.clone(), self.0))?;
		Ok(self.1.read(entry))
	}
}

pub trait ReadKdl<Input> {
	type Output;
	fn read(self, input: Input) -> Self::Output;
}

impl<'reader, 'node, Context> ReadKdl<&'reader mut Node<'node, Context>> for Peak<Value> {
	type Output = Result<&'node kdl::KdlValue, MissingEntryValue>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.get_entry(input)
	}
}
impl<'reader, 'node, Context> ReadKdl<&'reader mut Node<'node, Context>> for Peak<Typed<Value>> {
	type Output = Result<(Result<&'node str, MissingEntryType>, &'node kdl::KdlValue), MissingEntryValue>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.get_entry(input)
	}
}
impl<'reader, 'node, Context, V> ReadKdl<&'reader mut Node<'node, Context>> for Peak<FromValue<V>> where V: FromKdlValue {
	type Output = Result<V, crate::error::RequiredValue<V::Error>>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.get_entry(input) {
			Ok(Ok(value)) => Ok(value),
			Ok(Err(v_error)) => Err(crate::error::RequiredValue::Parse(v_error)),
			Err(missing) => Err(crate::error::RequiredValue::Missing(missing)),
		}
	}
}
impl<'reader, 'node, Context, V> ReadKdl<&'reader mut Node<'node, Context>> for Peak<Typed<FromValue<V>>> where V: FromKdlValue {
	type Output = Result<(Result<&'node str, MissingEntryType>, V), crate::error::RequiredValue<V::Error>>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.get_entry(input) {
			Ok((typed, Ok(value))) => Ok((typed, value)),
			Ok((_ty, Err(v_error))) => Err(crate::error::RequiredValue::Parse(v_error)),
			Err(missing) => Err(crate::error::RequiredValue::Missing(missing)),
		}
	}
}

impl<'reader, 'node, Context> ReadKdl<&'reader mut Node<'node, Context>> for Next<Value> {
	type Output = Result<&'node kdl::KdlValue, MissingEntryValue>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.get_entry(input)
	}
}
impl<'reader, 'node, Context> ReadKdl<&'reader mut Node<'node, Context>> for Next<Typed<Value>> {
	type Output = Result<(Result<&'node str, MissingEntryType>, &'node kdl::KdlValue), MissingEntryValue>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.get_entry(input)
	}
}
impl<'reader, 'node, Context, V> ReadKdl<&'reader mut Node<'node, Context>> for Next<FromValue<V>> where V: FromKdlValue {
	type Output = Result<V, crate::error::RequiredValue<V::Error>>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.get_entry(input) {
			Ok(Ok(value)) => Ok(value),
			Ok(Err(v_error)) => Err(crate::error::RequiredValue::Parse(v_error)),
			Err(missing) => Err(crate::error::RequiredValue::Missing(missing)),
		}
	}
}
impl<'reader, 'node, Context, V> ReadKdl<&'reader mut Node<'node, Context>> for Next<Typed<FromValue<V>>> where V: FromKdlValue {
	type Output = Result<(Result<&'node str, MissingEntryType>, V), crate::error::RequiredValue<V::Error>>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.get_entry(input) {
			Ok((typed, Ok(value))) => Ok((typed, value)),
			Ok((_ty, Err(v_error))) => Err(crate::error::RequiredValue::Parse(v_error)),
			Err(missing) => Err(crate::error::RequiredValue::Missing(missing)),
		}
	}
}

impl<'reader, 'node, Context, K: AsRef<str>> ReadKdl<&'reader mut Node<'node, Context>> for Property<K, Value> {
	type Output = Result<&'node kdl::KdlValue, MissingEntryValue>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.get_entry(input)
	}
}
impl<'reader, 'node, Context, K: AsRef<str>> ReadKdl<&'reader mut Node<'node, Context>> for Property<K, Typed<Value>> {
	type Output = Result<(Result<&'node str, MissingEntryType>, &'node kdl::KdlValue), MissingEntryValue>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		self.get_entry(input)
	}
}
impl<'reader, 'node, Context, K: AsRef<str>, V> ReadKdl<&'reader mut Node<'node, Context>> for Property<K, FromValue<V>> where V: FromKdlValue {
	type Output = Result<V, crate::error::RequiredValue<V::Error>>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.get_entry(input) {
			Ok(Ok(value)) => Ok(value),
			Ok(Err(v_error)) => Err(crate::error::RequiredValue::Parse(v_error)),
			Err(missing) => Err(crate::error::RequiredValue::Missing(missing)),
		}
	}
}
impl<'reader, 'node, Context, K: AsRef<str>, V> ReadKdl<&'reader mut Node<'node, Context>> for Property<K, Typed<FromValue<V>>> where V: FromKdlValue {
	type Output = Result<(Result<&'node str, MissingEntryType>, V), crate::error::RequiredValue<V::Error>>;
	fn read(self, input: &'reader mut Node<'node, Context>) -> Self::Output {
		match self.get_entry(input) {
			Ok((typed, Ok(value))) => Ok((typed, value)),
			Ok((_ty, Err(v_error))) => Err(crate::error::RequiredValue::Parse(v_error)),
			Err(missing) => Err(crate::error::RequiredValue::Missing(missing)),
		}
	}
}

impl<'doc, Context, T, RefOutput> ReadKdl<Node<'doc, Context>> for T where T: for<'reader> ReadKdl<&'reader mut Node<'doc, Context>, Output=RefOutput> {
	type Output = RefOutput;
	fn read(self, mut input: Node<'doc, Context>) -> Self::Output {
		self.read(&mut input)
	}
}


#[derive(Clone)]
pub struct Typed<T>(pub T);
impl<'node, T> ReadKdl<&'node kdl::KdlEntry> for Typed<T>
where
	T: ReadKdl<&'node kdl::KdlEntry>,
{
	type Output = (
		Result<&'node str, MissingEntryType>,
		T::Output,
	);
	fn read(self, input: &'node kdl::KdlEntry) -> Self::Output {
		let identifier = input.ty().ok_or(MissingEntryType(input.clone()));
		let identifier = identifier.map(|id| id.value());
		let value = self.0.read(input);
		(identifier, value)
	}
}

pub struct Value;
impl<'node> ReadKdl<&'node kdl::KdlEntry> for Value
{
	type Output = &'node kdl::KdlValue;
	fn read(self, input: &'node kdl::KdlEntry) -> Self::Output {
		input.value()
	}
}

pub struct FromValue<T>(PhantomData<T>);
impl<T> FromValue<T> {
	pub fn new() -> Self {
		Self(Default::default())
	}
}
impl<T> Clone for FromValue<T> {
	fn clone(&self) -> Self {
		Self::new()
	}
}
impl<'doc, T> ReadKdl<&'doc kdl::KdlEntry> for FromValue<T>
where
	T: FromKdlValue,
{
	type Output = Result<T, <T as FromKdlValue>::Error>;
	fn read(self, input: &'doc kdl::KdlEntry) -> Self::Output {
		T::from_kdl(input.value())
	}
}

pub struct IterReadChildNodes<'reader, 'doc, Context, T>(IterChildNodesWithName<'reader, 'doc, Context>, T);
impl<'reader, 'doc, Context, T> Iterator for IterReadChildNodes<'reader, 'doc, Context, T>
where
	T: ReadKdl<Node<'doc, Context>> + Clone,
{
	type Item = T::Output;
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(reader) = self.0.next() {
			return Some(self.1.clone().read(reader));
		}
		None
	}
}

#[derive(Clone)]
pub struct Children<K, T>(pub K, pub T);
impl<'reader, 'doc, Context, K, T> ReadKdl<&'reader mut Node<'doc, Context>> for Children<K, T>
where
	K: Into<kdl::KdlIdentifier>,
	T: ReadKdl<Node<'doc, Context>> + Clone,
{
	type Output = IterReadChildNodes<'reader, 'doc, Context, T>;
	fn read(self, input: &'reader mut Node<'doc, Context>) -> Self::Output {
		IterReadChildNodes(IterChildNodesWithName(IterChildNodes(input, 0), self.0.into()), self.1)
	}
}

#[derive(Clone)]
pub struct Child<K, T>(pub K, pub T);
impl<'reader, 'doc, Context, K, T> ReadKdl<&'reader mut Node<'doc, Context>> for Child<K, T>
where
	K: Into<kdl::KdlIdentifier>,
	T: ReadKdl<Node<'doc, Context>>,
{
	type Output = Result<T::Output, MissingChild>;
	fn read(self, input: &'reader mut Node<'doc, Context>) -> Self::Output {
		let name = self.0.into();
		let missing = MissingChild(input.node.clone(), name.clone());
		let mut iter = IterChildNodesWithName(IterChildNodes(input, 0), name);
		let child = iter.next().ok_or(missing)?;
		Ok(self.1.read(child))
	}
}

#[derive(Clone)]
pub struct Reader;
impl<'node, Context> ReadKdl<Node<'node, Context>> for Reader {
	type Output = Node<'node, Context>;
	fn read(self, input: Node<'node, Context>) -> Self::Output {
		input
	}
}

pub struct FromNode<T>(PhantomData<T>);
impl<T> FromNode<T> {
	pub fn new() -> Self {
		Self(Default::default())
	}
}
impl<T> Clone for FromNode<T> {
	fn clone(&self) -> Self {
		Self::new()
	}
}
impl<'reader, 'node, Context, T> ReadKdl<&'reader mut Node<'node, Context>> for FromNode<T>
where
	T: FromKdlNode<Context>,
{
	type Output = Result<T, T::Error>;
	fn read(self, input: &'reader mut Node<Context>) -> Self::Output {
		T::from_kdl(input)
	}
}
