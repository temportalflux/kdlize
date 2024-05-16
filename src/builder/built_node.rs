use crate::{AsKdl, OmitIfEmpty};

pub struct BuiltNode {
	node: Option<kdl::KdlNode>,
	omit_if_empty: Option<OmitIfEmpty>,
}

impl Into<Option<kdl::KdlNode>> for BuiltNode {
	fn into(self) -> Option<kdl::KdlNode> {
		let node = self.node?;
		if self.omit_if_empty.is_none() {
			return Some(node);
		}
		let has_children = node.children().map(|doc| !doc.nodes().is_empty()).unwrap_or(false);
		let is_empty = node.entries().is_empty() && !has_children;
		(!is_empty).then_some(node)
	}
}

impl From<(Option<kdl::KdlNode>, Option<OmitIfEmpty>)> for BuiltNode {
	fn from((node, omit_if_empty): (Option<kdl::KdlNode>, Option<OmitIfEmpty>)) -> Self {
		Self { node, omit_if_empty }
	}
}

impl From<Option<kdl::KdlNode>> for BuiltNode {
	fn from(node: Option<kdl::KdlNode>) -> Self {
		Self::from((node, None))
	}
}

impl From<(Option<kdl::KdlNode>, OmitIfEmpty)> for BuiltNode {
	fn from((node, omit_if_empty): (Option<kdl::KdlNode>, OmitIfEmpty)) -> Self {
		Self::from((node, Some(omit_if_empty)))
	}
}

impl From<kdl::KdlNode> for BuiltNode {
	fn from(node: kdl::KdlNode) -> Self {
		Self::from((Some(node), None))
	}
}

impl From<(kdl::KdlNode, OmitIfEmpty)> for BuiltNode {
	fn from((node, omit_if_empty): (kdl::KdlNode, OmitIfEmpty)) -> Self {
		Self::from((Some(node), Some(omit_if_empty)))
	}
}

impl<Name> From<(Name, super::NodeBuilder)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
{
	fn from((name, builder): (Name, super::NodeBuilder)) -> Self {
		let node = builder.build(name);
		Self {
			node: Some(node),
			omit_if_empty: None,
		}
	}
}

impl<Name, Buildable> From<(Name, Option<Buildable>, Option<OmitIfEmpty>)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable, omit_if_empty): (Name, Option<Buildable>, Option<OmitIfEmpty>)) -> Self {
		let node = buildable.map(|buildable| buildable.as_kdl().build(name));
		Self { node, omit_if_empty }
	}
}

impl<Name, Buildable> From<(Name, &Option<Buildable>, Option<OmitIfEmpty>)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable, omit_if_empty): (Name, &Option<Buildable>, Option<OmitIfEmpty>)) -> Self {
		Self::from((name, buildable.as_ref(), omit_if_empty))
	}
}

impl<Name, Buildable> From<(Name, Option<Buildable>)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable): (Name, Option<Buildable>)) -> Self {
		Self::from((name, buildable, None))
	}
}

impl<Name, Buildable> From<(Name, &Option<Buildable>)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable): (Name, &Option<Buildable>)) -> Self {
		Self::from((name, buildable.as_ref(), None))
	}
}

impl<Name, Buildable> From<(Name, Option<Buildable>, OmitIfEmpty)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable, omit_if_empty): (Name, Option<Buildable>, OmitIfEmpty)) -> Self {
		Self::from((name, buildable, Some(omit_if_empty)))
	}
}

impl<Name, Buildable> From<(Name, &Option<Buildable>, OmitIfEmpty)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable, omit_if_empty): (Name, &Option<Buildable>, OmitIfEmpty)) -> Self {
		Self::from((name, buildable.as_ref(), omit_if_empty))
	}
}

impl<Name, Buildable> From<(Name, Buildable, Option<OmitIfEmpty>)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable, omit_if_empty): (Name, Buildable, Option<OmitIfEmpty>)) -> Self {
		Self::from((name, Some(buildable), omit_if_empty))
	}
}

impl<Name, Buildable> From<(Name, Buildable)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable): (Name, Buildable)) -> Self {
		Self::from((name, Some(buildable), None))
	}
}

impl<Name, Buildable> From<(Name, Buildable, OmitIfEmpty)> for BuiltNode
where
	Name: Into<kdl::KdlIdentifier>,
	Buildable: AsKdl,
{
	fn from((name, buildable, omit_if_empty): (Name, Buildable, OmitIfEmpty)) -> Self {
		Self::from((name, Some(buildable), Some(omit_if_empty)))
	}
}

pub struct BuiltNodeList(Vec<BuiltNode>);

impl IntoIterator for BuiltNodeList {
	type Item = BuiltNode;

	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl From<Vec<BuiltNode>> for BuiltNodeList {
	fn from(value: Vec<BuiltNode>) -> Self {
		Self(value)
	}
}

impl<Key, Iterable> From<(Key, Iterable, Option<OmitIfEmpty>)> for BuiltNodeList
where
	Key: Into<kdl::KdlIdentifier>,
	Iterable: IntoIterator,
	Iterable::Item: AsKdl,
{
	fn from((key, iterable, omit_if_empty): (Key, Iterable, Option<OmitIfEmpty>)) -> Self {
		let key: kdl::KdlIdentifier = key.into();
		Self(
			iterable
				.into_iter()
				.map(|item| BuiltNode::from((key.clone(), item, omit_if_empty)))
				.collect(),
		)
	}
}

impl<Key, Iterable> From<(Key, Iterable)> for BuiltNodeList
where
	Key: Into<kdl::KdlIdentifier>,
	Iterable: IntoIterator,
	Iterable::Item: AsKdl,
{
	fn from((key, iterable): (Key, Iterable)) -> Self {
		Self::from((key, iterable, None))
	}
}

impl<Key, Iterable> From<(Key, Iterable, OmitIfEmpty)> for BuiltNodeList
where
	Key: Into<kdl::KdlIdentifier>,
	Iterable: IntoIterator,
	Iterable::Item: AsKdl,
{
	fn from((key, iterable, omit_if_empty): (Key, Iterable, OmitIfEmpty)) -> Self {
		Self::from((key, iterable, Some(omit_if_empty)))
	}
}
