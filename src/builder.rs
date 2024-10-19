mod built_node;
pub use built_node::*;

#[derive(Clone, Copy)]
pub struct OmitIfEmpty;

#[derive(Default, Debug)]
pub struct NodeBuilder {
	entries: Vec<kdl::KdlEntry>,
	children: Vec<kdl::KdlNode>,
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

		node.clear_fmt_recursive();
		node.fmt();

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
	fn set_type(&mut self, desired_type: Option<kdl::KdlIdentifier>) {
		let Some(entry) = self.entries.get_mut(0) else { return };
		match (entry.ty(), desired_type) {
			(None, None) => return,
			(Some(existing), Some(desired)) if *existing == desired => return,
			(Some(_), None) => {
				// must re-create the entry since there is no API in kdl for removing the type of an entry
				*entry = match entry.name() {
					None => kdl::KdlEntry::new(entry.value().clone()),
					Some(name) => kdl::KdlEntry::new_prop(name.clone(), entry.value().clone()),
				};
			}
			(None | Some(_), Some(type_id)) => {
				entry.set_ty(type_id);
			}
		}
	}

	pub fn with_type<TypeId>(mut self, ty: TypeId) -> Self
	where
		TypeId: Into<kdl::KdlIdentifier>,
	{
		self.set_type(Some(ty.into()));
		self
	}

	pub fn without_type(mut self) -> Self {
		self.set_type(None);
		self
	}

	pub fn entry(&mut self, entry: impl Into<kdl::KdlEntry>) {
		self.entries.push(entry.into());
	}

	pub fn with_entry(mut self, entry: impl Into<kdl::KdlEntry>) -> Self {
		self.entry(entry);
		self
	}

	pub fn entry_typed(&mut self, ty: impl Into<kdl::KdlIdentifier>, entry: impl Into<kdl::KdlEntry>) {
		self.entries.push({
			let mut entry: kdl::KdlEntry = entry.into();
			entry.set_ty(ty);
			entry
		});
	}

	pub fn with_entry_typed(mut self, entry: impl Into<kdl::KdlEntry>, ty: impl Into<kdl::KdlIdentifier>) -> Self {
		self.entry_typed(ty, entry);
		self
	}

	pub fn child(&mut self, child: impl Into<BuiltNode>) {
		let node: BuiltNode = child.into();
		if let Some(node) = node.into() {
			self.children.push(node);
		}
	}

	pub fn children(&mut self, iter: impl Into<BuiltNodeList>) {
		let list: BuiltNodeList = iter.into();
		for built_node in list.into_iter() {
			self.child(built_node);
		}
	}

	pub fn with(mut self, other: impl Into<Self>) -> Self {
		self += other.into();
		self
	}
}

#[cfg(test)]
mod test {
	use super::*;

	static NODE_NAME: &'static str = "doc";

	fn built_empty() -> kdl::KdlNode {
		kdl::KdlNode::new(NODE_NAME)
	}

	fn built_child_empty() -> kdl::KdlNode {
		let mut node = kdl::KdlNode::new(NODE_NAME);
		node.ensure_children().nodes_mut().push(kdl::KdlNode::new("node"));
		node
	}

	fn built_nonempty() -> kdl::KdlNode {
		built_children(vec![{
			let mut node = kdl::KdlNode::new("node");
			node.entries_mut().push(kdl::KdlEntry::new("content"));
			node
		}])
	}

	fn built_children(nodes: Vec<kdl::KdlNode>) -> kdl::KdlNode {
		let mut node = kdl::KdlNode::new(NODE_NAME);
		*node.ensure_children().nodes_mut() = nodes;
		node
	}

	fn node_empty() -> kdl::KdlNode {
		kdl::KdlNode::new("node")
	}

	fn node_nonempty() -> kdl::KdlNode {
		let mut node = kdl::KdlNode::new("node");
		node.entries_mut().push(kdl::KdlEntry::new(typed_nonempty()));
		node
	}

	fn typed_empty() -> String {
		String::new()
	}

	fn typed_nonempty() -> String {
		"content".to_owned()
	}

	mod child {
		use super::*;

		mod node {
			use super::*;

			#[test]
			fn node() {
				let mut builder = NodeBuilder::default();
				builder.child(node_empty());
				assert_eq!(builder.build(NODE_NAME), built_child_empty());
			}

			#[test]
			fn node_omitable_empty() {
				let mut builder = NodeBuilder::default();
				builder.child((node_empty(), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());
			}

			#[test]
			fn node_omitable_nonempty() {
				let mut builder = NodeBuilder::default();
				builder.child((node_nonempty(), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_nonempty());
			}

			#[test]
			fn node_opt_none() {
				let mut builder = NodeBuilder::default();
				builder.child(None::<kdl::KdlNode>);
				assert_eq!(builder.build(NODE_NAME), built_empty());
			}

			#[test]
			fn node_opt_some() {
				let mut builder = NodeBuilder::default();
				builder.child(Some(node_empty()));
				assert_eq!(builder.build(NODE_NAME), built_child_empty());
			}

			#[test]
			fn node_omitable_opt_none() {
				let mut builder = NodeBuilder::default();
				builder.child((None::<kdl::KdlNode>, OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());
			}

			#[test]
			fn node_omitable_opt_some_empty() {
				let mut builder = NodeBuilder::default();
				builder.child((Some(node_empty()), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());
			}

			#[test]
			fn node_omitable_opt_some_nonempty() {
				let mut builder = NodeBuilder::default();
				builder.child((Some(node_nonempty()), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_nonempty());
			}
		}

		mod typed {
			use super::*;

			#[test]
			fn typed() {
				let mut builder = NodeBuilder::default();
				builder.child(("node", &typed_empty()));
				assert_eq!(builder.build(NODE_NAME), built_child_empty());

				let mut builder = NodeBuilder::default();
				builder.child(("node", typed_empty()));
				assert_eq!(builder.build(NODE_NAME), built_child_empty());
			}

			#[test]
			fn typed_omitable_empty() {
				let mut builder = NodeBuilder::default();
				builder.child(("node", &typed_empty(), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());

				let mut builder = NodeBuilder::default();
				builder.child(("node", typed_empty(), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());
			}

			#[test]
			fn typed_omitable_nonempty() {
				let mut builder = NodeBuilder::default();
				builder.child(("node", &typed_nonempty(), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_nonempty());

				let mut builder = NodeBuilder::default();
				builder.child(("node", typed_nonempty(), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_nonempty());
			}

			#[test]
			fn typed_opt_none() {
				let mut builder = NodeBuilder::default();
				builder.child(("node", None::<&String>));
				assert_eq!(builder.build(NODE_NAME), built_empty());

				let mut builder = NodeBuilder::default();
				builder.child(("node", None::<String>));
				assert_eq!(builder.build(NODE_NAME), built_empty());
			}

			#[test]
			fn typed_opt_some() {
				let mut builder = NodeBuilder::default();
				builder.child(("node", Some(&typed_empty())));
				assert_eq!(builder.build(NODE_NAME), built_child_empty());

				let mut builder = NodeBuilder::default();
				builder.child(("node", Some(typed_empty())));
				assert_eq!(builder.build(NODE_NAME), built_child_empty());
			}

			#[test]
			fn typed_omitable_opt_none() {
				let mut builder = NodeBuilder::default();
				builder.child(("node", None::<&String>, OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());

				let mut builder = NodeBuilder::default();
				builder.child(("node", None::<String>, OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());
			}

			#[test]
			fn typed_omitable_opt_some_empty() {
				let mut builder = NodeBuilder::default();
				builder.child(("node", Some(&typed_empty()), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());

				let mut builder = NodeBuilder::default();
				builder.child(("node", Some(typed_empty()), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_empty());
			}

			#[test]
			fn typed_omitable_opt_some_nonempty() {
				let mut builder = NodeBuilder::default();
				builder.child(("node", Some(&typed_nonempty()), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_nonempty());

				let mut builder = NodeBuilder::default();
				builder.child(("node", Some(typed_nonempty()), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_nonempty());
			}
		}
	}

	mod children {
		use super::*;

		#[test]
		fn nodes() {
			let mut builder = NodeBuilder::default();
			builder.children(vec![BuiltNode::from(node_nonempty()), BuiltNode::from(node_nonempty())]);
			assert_eq!(
				builder.build(NODE_NAME),
				built_children(vec![node_nonempty(), node_nonempty(),])
			);
		}

		mod vec {
			use super::*;

			fn list() -> Vec<String> {
				vec![typed_empty(), typed_nonempty()]
			}

			fn expected() -> kdl::KdlNode {
				built_children(vec![node_empty(), node_nonempty()])
			}

			#[test]
			fn by_ref() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", &list()));
				assert_eq!(builder.build(NODE_NAME), expected());
			}

			#[test]
			fn by_value() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", list()));
				assert_eq!(builder.build(NODE_NAME), expected());
			}

			#[test]
			fn as_iter() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", list().iter()));
				assert_eq!(builder.build(NODE_NAME), expected());
			}

			#[test]
			fn into_iter() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", list().into_iter()));
				assert_eq!(builder.build(NODE_NAME), expected());
			}

			#[test]
			fn omit_empty() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", &list(), OmitIfEmpty));
				assert_eq!(builder.build(NODE_NAME), built_nonempty());
			}
		}

		mod map {
			use super::*;
			use std::collections::BTreeMap;

			fn data() -> BTreeMap<&'static str, String> {
				[("empty", typed_empty()), ("nonempty", typed_nonempty())].into()
			}

			fn expected() -> kdl::KdlNode {
				built_children(vec![
					{
						let mut node = kdl::KdlNode::new("node");
						node.entries_mut().push(kdl::KdlEntry::new("empty"));
						node
					},
					{
						let mut node = kdl::KdlNode::new("node");
						node.entries_mut().push(kdl::KdlEntry::new("nonempty"));
						node.entries_mut().push(kdl::KdlEntry::new(typed_nonempty()));
						node
					},
				])
			}

			#[test]
			fn by_ref() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", &data()));
				assert_eq!(builder.build(NODE_NAME), expected());
			}

			#[test]
			fn by_value() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", data()));
				assert_eq!(builder.build(NODE_NAME), expected());
			}

			#[test]
			fn as_iter() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", data().iter()));
				assert_eq!(builder.build(NODE_NAME), expected());
			}

			#[test]
			fn into_iter() {
				let mut builder = NodeBuilder::default();
				builder.children(("node", data().into_iter()));
				assert_eq!(builder.build(NODE_NAME), expected());
			}
		}
	}
}
