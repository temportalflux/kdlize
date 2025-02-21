use super::{Entry, OmitIfEmpty, OmitIfEqual, Property, Typed, Value};
use crate::{AsKdlNode, AsKdlValue};

#[derive(Default, Debug, Clone)]
pub struct Node {
	pub(super) entries: Vec<kdl::KdlEntry>,
	pub(super) children: Vec<kdl::KdlNode>,
}

impl Node {
	pub fn is_empty(&self) -> bool {
		self.entries.is_empty() && self.children.is_empty()
	}

	pub fn into_document(self) -> kdl::KdlDocument {
		let mut doc = kdl::KdlDocument::new();
		*doc.nodes_mut() = self.children;
		doc
	}

	pub fn build(self, name: impl Into<kdl::KdlIdentifier>) -> kdl::KdlNode {
		let Self {
			mut entries,
			mut children,
		} = self;
		let mut node = kdl::KdlNode::new(name);

		node.entries_mut().reserve(entries.len());

		// Push all of the unnamed/values first
		// unstable:drain_filter
		let mut i = 0;
		while i < entries.len() {
			let is_valid = !entries[i].value().is_null();
			if entries[i].name().is_none() || !is_valid {
				let val = entries.remove(i);
				if is_valid {
					node.entries_mut().push(val);
				}
			} else {
				i += 1;
			}
		}
		// Then push all of the named properties
		if !entries.is_empty() {
			node.entries_mut().append(&mut entries);
		}

		if !children.is_empty() {
			node.ensure_children().nodes_mut().append(&mut children);
		}

		node.autoformat();

		node
	}
}

impl Into<kdl::KdlDocument> for Node {
	fn into(self) -> kdl::KdlDocument {
		self.into_document()
	}
}

impl Node {
	pub fn push(&mut self, component: impl NodeComponent) {
		component.apply_to(self);
	}

	pub fn with(mut self, component: impl NodeComponent) -> Self {
		self.push(component);
		self
	}

	pub fn with_type(mut self, ty: impl Into<kdl::KdlIdentifier>) -> Self {
		if let Some(entry) = self.entries.get_mut(0) {
			entry.set_ty(ty);
		}
		self
	}
}

impl<T: NodeComponent> std::ops::Add<T> for Node {
	type Output = Self;
	fn add(self, component: T) -> Self::Output {
		self.with(component)
	}
}
impl<T: NodeComponent> std::ops::AddAssign<T> for Node {
	fn add_assign(&mut self, component: T) {
		self.push(component);
	}
}

pub trait NodeComponent {
	fn apply_to(self, builder: &mut Node);
}
impl NodeComponent for kdl::KdlNode {
	fn apply_to(self, builder: &mut Node) {
		builder.children.push(self);
	}
}
impl NodeComponent for Node {
	fn apply_to(mut self, builder: &mut Node) {
		builder.entries.append(&mut self.entries);
		builder.children.append(&mut self.children);
	}
}
impl<V: AsKdlNode> NodeComponent for &V {
	fn apply_to(self, builder: &mut Node) {
		*builder += self.as_kdl();
	}
}

pub trait IntoNodeBuilder {
	fn into_node(self) -> Node;
}
impl IntoNodeBuilder for Node {
	fn into_node(self) -> Node {
		self
	}
}
impl<V: AsKdlNode> IntoNodeBuilder for &V {
	fn into_node(self) -> Node {
		self.as_kdl()
	}
}
impl<V: AsKdlNode> IntoNodeBuilder for Option<&V> {
	fn into_node(self) -> Node {
		self.as_kdl()
	}
}
impl IntoNodeBuilder for Entry {
	fn into_node(self) -> Node {
		Node::default().with(self)
	}
}
impl NodeComponent for Entry {
	fn apply_to(self, builder: &mut Node) {
		if !self.entry.value().is_null() {
			builder.entries.push(self.entry);
		}
	}
}
impl<V: Into<Entry>> NodeComponent for OmitIfEmpty<V> {
	fn apply_to(self, builder: &mut Node) {
		let entry_builder: Entry = self.0.into();
		if let kdl::KdlValue::String(inner) = entry_builder.entry.value() {
			if inner.is_empty() {
				return;
			}
		}
		entry_builder.apply_to(builder);
	}
}
impl<V: Into<Entry>> IntoNodeBuilder for OmitIfEmpty<V> {
	fn into_node(self) -> Node {
		let entry_builder: Entry = self.0.into();
		if let kdl::KdlValue::String(inner) = entry_builder.entry.value() {
			if inner.is_empty() {
				return Node::default();
			}
		}
		entry_builder.into_node()
	}
}
impl<V: AsKdlNode> NodeComponent for OmitIfEmpty<&Option<V>> {
	fn apply_to(self, builder: &mut Node) {
		let node = self.0.as_ref().as_kdl();
		if !node.is_empty() {
			*builder += node;
		}
	}
}
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlNode> NodeComponent for OmitIfEmpty<Typed<Ty, &Option<V>>> {
	fn apply_to(self, builder: &mut Node) {
		let mut node = Node::default();
		self.0.apply_to(&mut node);
		if !node.is_empty() {
			*builder += node;
		}
	}
}
impl<V, T> NodeComponent for OmitIfEqual<V, T>
where
	V: super::InnerValue<Inner = T> + Into<Entry>,
	T: PartialEq + AsKdlValue,
{
	fn apply_to(self, builder: &mut Node) {
		if self.0.inner() != &self.1 {
			let entry_builder: Entry = self.0.into();
			entry_builder.apply_to(builder);
		}
	}
}
impl<V, T> IntoNodeBuilder for OmitIfEqual<V, T>
where
	V: super::InnerValue<Inner = T> + Into<Entry>,
	T: PartialEq + AsKdlValue,
{
	fn into_node(self) -> Node {
		if self.0.inner() != &self.1 {
			let entry_builder: Entry = self.0.into();
			entry_builder.into_node()
		} else {
			Node::default()
		}
	}
}
impl<V: AsKdlValue> IntoNodeBuilder for Value<V> {
	fn into_node(self) -> Node {
		let entry: Entry = self.into();
		entry.into_node()
	}
}
impl<V: AsKdlValue> NodeComponent for Value<V> {
	fn apply_to(self, builder: &mut Node) {
		let entry: Entry = self.into();
		entry.apply_to(builder);
	}
}
impl NodeComponent for &kdl::KdlValue {
	fn apply_to(self, builder: &mut Node) {
		let mut entry = Entry::default();
		entry.entry.set_value(self.clone());
		entry.apply_to(builder);
	}
}
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlValue> IntoNodeBuilder for Typed<Ty, Value<V>> {
	fn into_node(self) -> Node {
		let entry: Entry = self.into();
		entry.into_node()
	}
}
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlValue> NodeComponent for Typed<Ty, Value<V>> {
	fn apply_to(self, builder: &mut Node) {
		let entry: Entry = self.into();
		entry.apply_to(builder);
	}
}
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlNode> NodeComponent for Typed<Ty, V> {
	fn apply_to(self, builder: &mut Node) {
		self.into_node().apply_to(builder);
	}
}
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlNode> IntoNodeBuilder for Typed<Ty, V> {
	fn into_node(self) -> Node {
		let mut node = self.1.as_kdl();
		if let Some(entry) = node.entries.get_mut(0) {
			entry.set_ty(self.0.into());
		}
		node
	}
}
impl<K: Into<kdl::KdlIdentifier>, V: Into<Entry>> IntoNodeBuilder for Property<K, V> {
	fn into_node(self) -> Node {
		let entry: Entry = self.into();
		entry.into_node()
	}
}
impl<K: Into<kdl::KdlIdentifier>, V: Into<Entry>> NodeComponent for Property<K, V> {
	fn apply_to(self, builder: &mut Node) {
		let entry: Entry = self.into();
		entry.apply_to(builder);
	}
}

pub struct Child<K: Into<kdl::KdlIdentifier>, V>(pub K, pub V);
impl<K: Into<kdl::KdlIdentifier>, V: IntoNodeBuilder> NodeComponent for Child<K, V> {
	fn apply_to(self, builder: &mut Node) {
		let child = self.1.into_node();
		builder.children.push(child.build(self.0));
	}
}

pub struct Children<K: Into<kdl::KdlIdentifier>, V>(pub K, pub V);
impl<K: Into<kdl::KdlIdentifier>, V> NodeComponent for Children<K, V>
where
	V: IntoIterator,
	V::Item: IntoNodeBuilder,
{
	fn apply_to(self, builder: &mut Node) {
		let node_name: kdl::KdlIdentifier = self.0.into();
		for item in self.1.into_iter() {
			let child = item.into_node();
			builder.children.push(child.build(node_name.clone()));
		}
	}
}
impl<K: Into<kdl::KdlIdentifier>, V> NodeComponent for Children<K, Value<V>>
where
	V: IntoIterator,
	V::Item: AsKdlValue,
{
	fn apply_to(self, builder: &mut Node) {
		let node_name: kdl::KdlIdentifier = self.0.into();
		for item in self.1 .0.into_iter() {
			let child = Node::default() + Value(item);
			builder.children.push(child.build(node_name.clone()));
		}
	}
}

impl<K: Into<kdl::KdlIdentifier>, V: IntoNodeBuilder> NodeComponent for OmitIfEmpty<Child<K, V>> {
	fn apply_to(self, builder: &mut Node) {
		let child = self.0 .1.into_node();
		if !child.is_empty() {
			builder.children.push(child.build(self.0 .0));
		}
	}
}
impl<K: Into<kdl::KdlIdentifier>, V> NodeComponent for OmitIfEmpty<Children<K, V>>
where
	V: IntoIterator,
	V::Item: AsKdlNode,
{
	fn apply_to(self, builder: &mut Node) {
		let node_name: kdl::KdlIdentifier = self.0 .0.into();
		for item in self.0 .1.into_iter() {
			let child = item.as_kdl();
			if !child.is_empty() {
				builder.children.push(child.build(node_name.clone()));
			}
		}
	}
}
impl<K: Into<kdl::KdlIdentifier>, V> NodeComponent for OmitIfEmpty<Children<K, Value<V>>>
where
	V: IntoIterator,
	V::Item: AsKdlValue,
{
	fn apply_to(self, builder: &mut Node) {
		let node_name: kdl::KdlIdentifier = self.0 .0.into();
		for item in self.0 .1 .0.into_iter() {
			let child = Node::default() + OmitIfEmpty(Value(item));
			println!("{child:?}");
			if !child.is_empty() {
				builder.children.push(child.build(node_name.clone()));
			}
		}
	}
}
