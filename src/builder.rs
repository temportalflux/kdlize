mod node;
pub use node::*;

mod entry;
pub use entry::*;

pub struct OmitIfEmpty<V>(pub V);

#[cfg(test)]
mod test {
	use super::*;
	use crate::AsKdlNode;

	#[test]
	fn node_with_value() {
		let value = Node::default().with(Value(42)).build("node");
		let expected = {
			let mut node = kdl::KdlNode::new("node");
			node.entries_mut().push(kdl::KdlEntry::new(42));
			node
		};
		assert_eq!(value.to_string(), expected.to_string());
	}

	#[test]
	fn node_with_prop() {
		let value = Node::default()
			.with(Entry::default().name("prop").value(42))
			.build("node");
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
			let mut node = Node::default();
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
			let mut node = Node::default();
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
			let mut node = Node::default();
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
			let mut node = Node::default();
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
			let mut node = Node::default();
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
			let mut node = Node::default();
			node.push(Child("child", {
				let mut node = Node::default();
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
			fn as_kdl(&self) -> Node {
				Node::default().with(Value(self.0))
			}
		}

		let value = {
			let mut node = Node::default();
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
			fn as_kdl(&self) -> Node {
				Node::default().with(Value(self.0))
			}
		}

		let value = {
			let mut node = Node::default();
			node.push(OmitIfEmpty(Child("opt_none", &None::<Example>)));
			node.push(OmitIfEmpty(Child("opt_some", Node::default().with(Value("abc")))));
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
			fn as_kdl(&self) -> Node {
				Node::default().with(Value(self.0))
			}
		}

		let value = {
			let mut node = Node::default();
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
