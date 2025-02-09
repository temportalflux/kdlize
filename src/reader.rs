use crate::{
	error::{MissingChild, MissingEntry, MissingEntryType, MissingEntryValue, QueryError},
	FromKdlNode, FromKdlValue,
};
use std::marker::PhantomData;

pub struct Node<'doc, Context> {
	node: &'doc kdl::KdlNode,
	ctx: &'doc Context,
	entry_cursor: std::rc::Rc<std::cell::RefCell<usize>>,
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
			entry_cursor: std::rc::Rc::new(std::cell::RefCell::new(0)),
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
pub struct IterChildNodes<'doc, Context>(Node<'doc, Context>, usize);
impl<'doc, Context> Iterator for IterChildNodes<'doc, Context> {
	type Item = Node<'doc, Context>;
	fn next(&mut self) -> Option<Self::Item> {
		let document = self.0.document()?;
		let node = document.nodes().get(self.1)?;
		self.1 += 1;
		Some(Node::<'doc, Context>::new(node, &self.0.ctx))
	}
}

pub struct IterChildNodesWithName<'doc, Context>(IterChildNodes<'doc, Context>, kdl::KdlIdentifier);
impl<'doc, Context> Iterator for IterChildNodesWithName<'doc, Context> {
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

pub struct Query<'doc, Input, Output>(std::rc::Rc<dyn Fn(Input) -> Output + 'doc>);
impl<'doc, Input, Output> Clone for Query<'doc, Input, Output> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}
impl<'doc, Input, Output> Query<'doc, Input, Output> {
	fn new<F: 'doc + Fn(Input) -> Output>(f: F) -> Self {
		Self(std::rc::Rc::new(f))
	}

	fn call(&self, input: Input) -> Output {
		(self.0)(input)
	}
}

struct IterQuery<'doc, Iter, Input, Output>(Iter, Query<'doc, Input, Output>);
impl<'doc, Iter, Input, Output> Iterator for IterQuery<'doc, Iter, Input, Output> where Iter: Iterator<Item=Input> {
	type Item = Output;
	fn next(&mut self) -> Option<Self::Item> {
		let input = self.0.next()?;
		let output = self.1.call(input);
		Some(output)
	}
}

/*
impl<'doc, Input, T, LError> Query<'doc, Input, Result<T, LError>> {
	fn join<Output, RError>(lhs: Self, rhs: Query<'doc, T, Result<Output, RError>>) -> Query<'doc, Input, Result<Output, QueryError>>
		where QueryError: From<LError> + From<RError>, Input: 'doc, Output: 'doc, T: 'doc, LError: 'doc, RError: 'doc {
		Query::new(move |input: Input| -> Result<Output, QueryError> {
			let value = lhs.call(input)?;
			Ok(rhs.call(value)?)
		})
	}
}
*/
impl<'doc, Input, Context> Query<'doc, Input, IterChildNodes<'doc, Context>>
where Input: 'doc, Context: 'doc {
	fn join<Output>(lhs: Self, rhs: Query<'doc, Node<'doc, Context>, Output>) -> Query<'doc, Input, IterQuery<'doc, IterChildNodes<'doc, Context>, Node<'doc, Context>, Output>>
	where Output: 'doc {
		Query::new(move |input: Input| -> IterQuery<'doc, IterChildNodes<'doc, Context>, Node<'doc, Context>, Output> {
			IterQuery(lhs.call(input), rhs.clone())
		})
	}
}

#[macro_export]
macro_rules! query {
	($q:expr) => ($q.as_query());
	($lhs:expr, $rhs:expr) => (
		Query::join(query!($lhs), query!($rhs))
	);
	($lhs:expr, $($rhz:expr),+) => (
		Query::join(query!($lhs), query!($($rhz),+))
	);
}

fn example() {
	let _: Query<'_, Node<'_, ()>, _> = query!(Peak);
	let _: Query<'_, Node<'_, ()>, _> = query!(Next);
	let _: Query<'_, Node<'_, ()>, _> = query!(Property("key"));

	let _: Query<'_, Node<'_, ()>, _> = query!(Peak, Entry);
	let _: Query<'_, Node<'_, ()>, _> = query!(Peak, Typed);
	let _: Query<'_, Node<'_, ()>, _> = query!(Peak, Value);
	let _: Query<'_, Node<'_, ()>, _> = query!(Next, Entry);
	let _: Query<'_, Node<'_, ()>, _> = query!(Next, Typed);
	let _: Query<'_, Node<'_, ()>, _> = query!(Next, Value);
	let _: Query<'_, Node<'_, ()>, _> = query!(Property("key"), Entry);
	let _: Query<'_, Node<'_, ()>, _> = query!(Property("key"), Typed);
	let _: Query<'_, Node<'_, ()>, _> = query!(Property("key"), Value);
	
	let _: Query<'_, Node<'_, ()>, _> = query!(Child("node"));
	let _: Query<'_, Node<'_, ()>, _> = query!(Child("node"), Next);
	let _: Query<'_, Node<'_, ()>, _> = query!(Child("node"), Next, Entry);
	let _: Query<'_, Node<'_, ()>, _> = query!(Child("node"), Property("key"));
	let _: Query<'_, Node<'_, ()>, _> = query!(Child("node"), Next, Value);
	let _: Query<'_, Node<'_, ()>, _> = query!(Child("node"), Property("key"), Value);
	
	let _: Query<'_, Node<'_, ()>, _> = query!(Child("node"), Child("c1"));
	
	let _: Query<'_, Node<'_, ()>, _> = query!(AllChildren);
	let _: Query<'_, Node<'_, ()>, _> = query!(AllChildren, Next);

	let _ = Query::join(AllChildren.as_query(), Next.as_query());

}

pub struct Peak;
pub struct Next;
pub struct Property<K: AsRef<str>>(pub K);

trait AsQuery<'doc, Input, Output> {
	fn as_query(self) -> Query<'doc, Input, Output>;
}

fn join_iter<'doc, Input, IterT, T, Output>(lhs: Query<'doc, Input, IterT>, rhs: Query<'doc, T, Output>) -> Query<'doc, Input, IterQuery<'doc, IterT, T, Output>>
	where Input: 'doc, Output: 'doc, T: 'doc, IterT: 'doc {
	Query::new(move |input: Input| -> IterQuery<'doc, IterT, T, Output> {
		IterQuery(lhs.call(input), rhs.clone())
	})
}

impl<'reader, 'doc, Context> AsQuery<'doc, Node<'doc, Context>, Result<&'doc kdl::KdlEntry, QueryError>> for Peak {
	fn as_query(self) -> Query<'doc, Node<'doc, Context>, Result<&'doc kdl::KdlEntry, QueryError>> {
		Query::new(|input: Node<'doc, Context>| -> Result<&'doc kdl::KdlEntry, QueryError> {
			let idx = *input.entry_cursor.borrow();
			let entry = input.node.entry(idx);
			Ok(entry.ok_or(MissingEntry::new_index(input.node.clone(), idx))?)
		})
	}
}
impl<'reader, 'doc, Context> AsQuery<'doc, Node<'doc, Context>, Result<&'doc kdl::KdlEntry, QueryError>> for Next {
	fn as_query(self) -> Query<'doc, Node<'doc, Context>, Result<&'doc kdl::KdlEntry, QueryError>> {
		Query::new(|input: Node<'doc, Context>| -> Result<&'doc kdl::KdlEntry, QueryError> {
			let idx = *input.entry_cursor.borrow();
			let entry = input.node.entry(idx);
			*input.entry_cursor.borrow_mut() += 1;
			Ok(entry.ok_or(MissingEntryValue::new_index(input.node.clone(), idx))?)
		})
	}
}
impl<'reader, 'doc, Context, K: AsRef<str> + 'static> AsQuery<'doc, Node<'doc, Context>, Result<&'doc kdl::KdlEntry, QueryError>> for Property<K> {
	fn as_query(self) -> Query<'doc, Node<'doc, Context>, Result<&'doc kdl::KdlEntry, QueryError>> {
		Query::new(move |input: Node<'doc, Context>| -> Result<&'doc kdl::KdlEntry, QueryError> {
			let key = self.0.as_ref();
			let entry = input.node.entry(key);
			Ok(entry.ok_or(MissingEntryValue::new_prop(input.node.clone(), key))?)
		})
	}
}

pub struct Entry;
pub struct Typed;
pub struct Value;

impl<'doc> AsQuery<'doc, &'doc kdl::KdlEntry, Result<(Result<&'doc str, MissingEntryType>, &'doc kdl::KdlValue), QueryError>> for Entry {
	fn as_query(self) -> Query<'doc, &'doc kdl::KdlEntry, Result<(Result<&'doc str, MissingEntryType>, &'doc kdl::KdlValue), QueryError>> {
		Query::new(|input: &'doc kdl::KdlEntry| -> Result<(Result<&'doc str, MissingEntryType>, &'doc kdl::KdlValue), QueryError> {
			let ty = input.ty().ok_or(MissingEntryType(input.clone()));
			let ty = ty.map(|id| id.value());
			let value = input.value();
			Ok((ty, value))
		})
	}
}
impl<'doc> AsQuery<'doc, &'doc kdl::KdlEntry, Result<&'doc str, QueryError>> for Typed {
	fn as_query(self) -> Query<'doc, &'doc kdl::KdlEntry, Result<&'doc str, QueryError>> {
		Query::new(|input: &'doc kdl::KdlEntry| -> Result<&'doc str, QueryError> {
			let identifier = input.ty().ok_or(MissingEntryType(input.clone()))?;
			Ok(identifier.value())
		})
	}
}
impl<'doc> AsQuery<'doc, &'doc kdl::KdlEntry, Result<&'doc kdl::KdlValue, QueryError>> for Value {
	fn as_query(self) -> Query<'doc, &'doc kdl::KdlEntry, Result<&'doc kdl::KdlValue, QueryError>> {
		Query::new(|input: &'doc kdl::KdlEntry| Ok(input.value()))
	}
}

pub struct Child<K: Into<kdl::KdlIdentifier>>(pub K);
impl<'doc, Context, K: Into<kdl::KdlIdentifier>> AsQuery<'doc, Node<'doc, Context>, Result<Node<'doc, Context>, QueryError>> for Child<K> {
	fn as_query(self) -> Query<'doc, Node<'doc, Context>, Result<Node<'doc, Context>, QueryError>> {
		let name = self.0.into();
		Query::new(move |input: Node<'doc, Context>| -> Result<Node<'doc, Context>, QueryError> {
			let missing = MissingChild(input.node.clone(), name.clone());
			let mut iter = IterChildNodesWithName(IterChildNodes(input, 0), name.clone());
			Ok(iter.next().ok_or(missing)?)
		})
	}
}

pub struct AllChildren;
impl<'doc, Context> AsQuery<'doc, Node<'doc, Context>, IterChildNodes<'doc, Context>> for AllChildren {
	fn as_query(self) -> Query<'doc, Node<'doc, Context>, IterChildNodes<'doc, Context>> {
		Query::new(move |input: Node<'doc, Context>| IterChildNodes(input, 0))
	}
}

pub struct Children<K: Into<kdl::KdlIdentifier>>(pub K);

pub struct FromValue;

pub struct FromNode;

mod v61 {
	use super::*;

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
}
