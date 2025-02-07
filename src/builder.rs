use crate::{AsKdlNode, AsKdlValue};

#[derive(Default, Debug)]
pub struct NodeBuilder {
	entries: Vec<kdl::KdlEntry>,
	children: Vec<kdl::KdlNode>,
	omit_if_empty: bool,
}

impl NodeBuilder {
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
			omit_if_empty: _,
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

		node.clear_format_recursive();
		node.autoformat();

		node
	}
}

impl Into<kdl::KdlDocument> for NodeBuilder {
	fn into(self) -> kdl::KdlDocument {
		self.into_document()
	}
}

impl std::ops::AddAssign for NodeBuilder {
	fn add_assign(&mut self, mut rhs: Self) {
		self.entries.append(&mut rhs.entries);
		self.children.append(&mut rhs.children);
	}
}

impl<TypeId> std::ops::AddAssign<(TypeId, Self)> for NodeBuilder
where
	TypeId: Into<kdl::KdlIdentifier>,
{
	fn add_assign(&mut self, (type_id, mut node): (TypeId, Self)) {
		if let Some(entry) = node.entries.get_mut(0) {
			entry.set_ty(type_id);
		}
		*self += node;
	}
}

impl NodeBuilder {
	pub fn push(&mut self, component: impl NodeBuilderComponent) {
		component.apply_to(self);
	}
	
	pub fn with(mut self, component: impl NodeBuilderComponent) -> Self {
		self.push(component);
		self
	}
}

#[derive(Debug)]
pub struct EntryBuilder {
	entry: kdl::KdlEntry,
	omit_if_empty: bool,
}
impl Default for EntryBuilder {
	fn default() -> Self {
		Self { entry: kdl::KdlEntry::new(kdl::KdlValue::Null), omit_if_empty: false }
	}
}
impl EntryBuilder {
	pub fn name(mut self, name: impl Into<kdl::KdlIdentifier>) -> Self {
		self.entry.set_name(Some(name));
		self
	}

	pub fn ty(mut self, ty: impl Into<kdl::KdlIdentifier>) -> Self {
		self.entry.set_ty(ty);
		self
	}

	pub fn value<V: AsKdlValue>(mut self, value: V) -> Self {
		self.entry.set_value(value.as_kdl());
		self
	}

	pub fn omit_if_empty(mut self) -> Self {
		self.omit_if_empty = true;
		self
	}
}

pub struct Value<V: AsKdlValue>(pub V);
impl<V: AsKdlValue> Into<EntryBuilder> for Value<V> {
	fn into(self) -> EntryBuilder {
		let mut builder = EntryBuilder::default();
		builder.entry.set_value(Some(self.0.as_kdl()));
		builder
	}
}

pub struct Typed<Ty: Into<kdl::KdlIdentifier>, V: AsKdlValue>(pub Ty, pub Value<V>);
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlValue> Into<EntryBuilder> for Typed<Ty, V> {
	fn into(self) -> EntryBuilder {
		let mut builder: EntryBuilder = self.1.into();
		builder.entry.set_ty(self.0);
		builder
	}
}

pub struct Property<K: Into<kdl::KdlIdentifier>, V: Into<EntryBuilder>>(pub K, pub V);
impl<K: Into<kdl::KdlIdentifier>, V: Into<EntryBuilder>> Into<EntryBuilder> for Property<K, V> {
	fn into(self) -> EntryBuilder {
		let mut builder: EntryBuilder = self.1.into();
		builder.entry.set_name(Some(self.0));
		builder
	}
}

pub struct OmitIfEmpty<V>(pub V);
impl<V: Into<EntryBuilder>> Into<EntryBuilder> for OmitIfEmpty<V> {
	fn into(self) -> EntryBuilder {
		let mut builder: EntryBuilder = self.0.into();
		builder.omit_if_empty = true;
		builder
	}
}

pub trait NodeBuilderComponent {
	fn apply_to(self, builder: &mut NodeBuilder);
}
impl<T: Into<EntryBuilder>> NodeBuilderComponent for T {
	fn apply_to(self, builder: &mut NodeBuilder) {
		let entry_builder: EntryBuilder = self.into();
		if !entry_builder.entry.value().is_null() || !entry_builder.omit_if_empty {
			builder.entries.push(entry_builder.entry);
		}
	}
}

pub trait IntoNodeBuilder {
	fn into_node(self) -> NodeBuilder;
}
impl IntoNodeBuilder for NodeBuilder {
	fn into_node(self) -> NodeBuilder {
		self
	}
}
impl<V: AsKdlNode> IntoNodeBuilder for &V {
	fn into_node(self) -> NodeBuilder {
		self.as_kdl()
	}
}
impl IntoNodeBuilder for EntryBuilder {
	fn into_node(self) -> NodeBuilder {
		NodeBuilder::default().with(self)
	}
}
impl<V: AsKdlValue> IntoNodeBuilder for Value<V> {
	fn into_node(self) -> NodeBuilder {
		NodeBuilder::default().with(self)
	}
}
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlValue> IntoNodeBuilder for Typed<Ty, V> {
	fn into_node(self) -> NodeBuilder {
		NodeBuilder::default().with(self)
	}
}
impl<K: Into<kdl::KdlIdentifier>, V: Into<EntryBuilder>> IntoNodeBuilder for Property<K, V> {
	fn into_node(self) -> NodeBuilder {
		NodeBuilder::default().with(self)
	}
}
impl<V: IntoNodeBuilder> IntoNodeBuilder for OmitIfEmpty<V> {
	fn into_node(self) -> NodeBuilder {
		let mut builder = self.0.into_node();
		builder.omit_if_empty = true;
		builder
	}
}

pub struct Child<K: Into<kdl::KdlIdentifier>, V>(pub K, pub V);
impl<K: Into<kdl::KdlIdentifier>, V: IntoNodeBuilder> NodeBuilderComponent for Child<K, V> {
	fn apply_to(self, builder: &mut NodeBuilder) {
		let child = self.1.into_node();
		if !child.is_empty() || !child.omit_if_empty {
			builder.children.push(child.build(self.0));
		}
	}
}

pub struct Children<K: Into<kdl::KdlIdentifier>, V>(pub K, pub V);
impl<K: Into<kdl::KdlIdentifier>, V> NodeBuilderComponent for Children<K, V> where V: IntoIterator, V::Item: AsKdlNode {
	fn apply_to(self, builder: &mut NodeBuilder) {
		let node_name: kdl::KdlIdentifier = self.0.into();
		for item in self.1.into_iter() {
			let child = item.as_kdl();
			if !child.is_empty() || !child.omit_if_empty {
				builder.children.push(child.build(node_name.clone()));
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn node_with_value() {
		let value = NodeBuilder::default().with(Value(42)).build("node");
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.entries_mut().push(kdl::KdlEntry::new(42));
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn node_with_prop() {
		let value = NodeBuilder::default().with(EntryBuilder::default().name("prop").value(42)).build("node");
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.entries_mut().push({
				let mut entry = kdl::KdlEntry::new(42);
				entry.set_name(Some("prop"));
				entry
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn node_with_prop_opt() {
		let value = {
			let mut node = NodeBuilder::default();
			node.push(OmitIfEmpty(Property("prop", Value(&None::<String>))));
			node.push(OmitIfEmpty(Property("prop", Value(&Some(42)))));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.entries_mut().push({
				let mut entry = kdl::KdlEntry::new(42);
				entry.set_name(Some("prop"));
				entry
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn child_untyped_value() {
		let value = {
			let mut node = NodeBuilder::default();
			node.push(Child("child", Value(42)));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.set_children({
				let mut doc = kdl::KdlDocument::new();
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push(kdl::KdlEntry::new(42));
					node
				});
				doc
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn child_typed_value() {
		let value = {
			let mut node = NodeBuilder::default();
			node.push(Child("child", Typed("number", Value(3.0))));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.set_children({
				let mut doc = kdl::KdlDocument::new();
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push({
						let mut entry = kdl::KdlEntry::new(3.0);
						entry.set_ty("number");
						entry
					});
					node
				});
				doc
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn child_untyped_prop() {
		let value = {
			let mut node = NodeBuilder::default();
			node.push(Child("child", Property("prop", Value(42))));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.set_children({
				let mut doc = kdl::KdlDocument::new();
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push({
						let mut entry = kdl::KdlEntry::new(42);
						entry.set_name(Some("prop"));
						entry
					});
					node
				});
				doc
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn child_typed_prop() {
		let value = {
			let mut node = NodeBuilder::default();
			node.push(Child("child", Property("prop", Typed("number", Value(3.0)))));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.set_children({
				let mut doc = kdl::KdlDocument::new();
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push({
						let mut entry = kdl::KdlEntry::new(3.0);
						entry.set_name(Some("prop"));
						entry.set_ty("number");
						entry
					});
					node
				});
				doc
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn child_nodebuilder() {
		let value = {
			let mut node = NodeBuilder::default();
			node.push(Child("child", {
				let mut node = NodeBuilder::default();
				node.push(Value(42));
				node
			}));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.set_children({
				let mut doc = kdl::KdlDocument::new();
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push(kdl::KdlEntry::new(42));
					node
				});
				doc
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn child_askdlnode() {
		struct Example(u32);
		impl AsKdlNode for Example {
			fn as_kdl(&self) -> NodeBuilder {
				NodeBuilder::default().with(Value(self.0))
			}
		}

		let value = {
			let mut node = NodeBuilder::default();
			node.push(Child("child", &Example(100)));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.set_children({
				let mut doc = kdl::KdlDocument::new();
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push(kdl::KdlEntry::new(100));
					node
				});
				doc
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}
	
	#[test]
	fn child_askdlnode_opt() {
		struct Example(u32);
		impl AsKdlNode for Example {
			fn as_kdl(&self) -> NodeBuilder {
				NodeBuilder::default().with(Value(self.0))
			}
		}

		let value = {
			let mut node = NodeBuilder::default();
			node.push(Child("opt_none", OmitIfEmpty(&None::<Example>)));
			node.push(Child("opt_some", OmitIfEmpty(NodeBuilder::default().with(Value("abc")))));
			node.push(Child("req_none", Value(&None::<String>)));
			node.push(Child("req_some", &Some(Example(100))));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.set_children({
				let mut doc = kdl::KdlDocument::new();
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("opt_some");
					node.entries_mut().push(kdl::KdlEntry::new("abc"));
					node
				});
				doc.nodes_mut().push(kdl::KdlNode::new("req_none"));
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("req_some");
					node.entries_mut().push(kdl::KdlEntry::new(100));
					node
				});
				doc
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn children_values() {
		struct Example(u32);
		impl AsKdlNode for Example {
			fn as_kdl(&self) -> NodeBuilder {
				NodeBuilder::default().with(Value(self.0))
			}
		}

		let value = {
			let mut node = NodeBuilder::default();
			node.push(Children("child", &vec![Example(3), Example(5), Example(2)]));
			node.build("node")
		};
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.set_children({
				let mut doc = kdl::KdlDocument::new();
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push(kdl::KdlEntry::new(3));
					node
				});
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push(kdl::KdlEntry::new(5));
					node
				});
				doc.nodes_mut().push({
					let mut node = kdl::KdlNode::new("child");
					node.entries_mut().push(kdl::KdlEntry::new(2));
					node
				});
				doc
			});
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}
}
