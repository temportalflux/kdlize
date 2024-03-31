use crate::AsKdl;

#[derive(Default)]
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
			if entries[i].name().is_none() {
				let val = entries.remove(i);
				node.entries_mut().push(val);
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

	pub fn set_first_entry_ty(&mut self, ty: impl Into<kdl::KdlIdentifier>) {
		if let Some(entry) = self.entries.get_mut(0) {
			entry.set_ty(ty);
		}
	}

	pub fn without_type(mut self) -> Self {
		if let Some(entry) = self.entries.get_mut(0) {
			if entry.ty().is_some() {
				*entry = match entry.name() {
					None => kdl::KdlEntry::new(entry.value().clone()),
					Some(name) => kdl::KdlEntry::new_prop(name.clone(), entry.value().clone()),
				};
			}
		}
		self
	}

	pub fn with_extension(mut self, node: NodeBuilder) -> Self {
		self += node;
		self
	}

	pub fn append_typed(&mut self, ty: impl Into<kdl::KdlIdentifier>, mut node: NodeBuilder) {
		node.set_first_entry_ty(ty);
		*self += node;
	}

	pub fn push_entry(&mut self, entry: impl Into<kdl::KdlEntry>) {
		self.entries.push(entry.into());
	}

	pub fn push_entry_typed(&mut self, entry: impl Into<kdl::KdlEntry>, ty: impl Into<kdl::KdlIdentifier>) {
		self.entries.push({
			let mut entry = entry.into();
			entry.set_ty(ty);
			entry
		});
	}

	pub fn with_entry(mut self, entry: impl Into<kdl::KdlEntry>) -> Self {
		self.push_entry(entry);
		self
	}

	pub fn with_entry_typed(mut self, entry: impl Into<kdl::KdlEntry>, ty: impl Into<kdl::KdlIdentifier>) -> Self {
		self.push_entry_typed(entry, ty);
		self
	}

	pub fn push_child_entry(&mut self, name: impl Into<kdl::KdlIdentifier>, entry: impl Into<kdl::KdlEntry>) {
		self.push_child(Self::default().with_entry(entry.into()).build(name));
	}

	pub fn push_child_entry_typed(
		&mut self,
		name: impl Into<kdl::KdlIdentifier>,
		ty: impl Into<kdl::KdlIdentifier>,
		entry: impl Into<kdl::KdlEntry>,
	) {
		self.push_child(Self::default().with_entry_typed(entry.into(), ty).build(name));
	}
}

impl std::ops::AddAssign for NodeBuilder {
	fn add_assign(&mut self, mut rhs: Self) {
		self.entries.append(&mut rhs.entries);
		self.children.append(&mut rhs.children);
	}
}

impl Into<kdl::KdlDocument> for NodeBuilder {
	fn into(self) -> kdl::KdlDocument {
		self.into_document()
	}
}

// Building children
impl NodeBuilder {
	/// Pushes a node into the list of children.
	/// ```
	/// let mut builder = NodeBuilder::default();
	///
	/// // pushes the node itself
	/// let node_with_content: kdl::KdlNode;
	/// builder.push_child(node_with_content);
	///
	/// // none, so no node is pushed
	/// let node_opt: Option<kdl::KdlNode> = None;
	/// builder.push_child(node_opt);
	///
	/// // the provided node is empty, and the OmitIfEmpty flag is provided,
	/// // so no node is pushed.
	/// let empty_node: kdl::KdlNode;
	/// builder.push_child((empty_node, OmitIfEmpty));
	///
	/// // node is some but is empty, so no node is pushed
	/// let some_empty_node: Option<kdl::KdlNode>;
	/// builder.push_child((some_empty_node, OmitIfEmpty));
	/// ```
	pub fn push_child(&mut self, child: impl Into<BuiltNode>) {
		let node: BuiltNode = child.into();
		if let Some(node) = node.into() {
			self.children.push(node);
		}
	}

	// Builds a node from a name and `AsKdl` implementation,
	// pushing the generated node into the list of children.
	/// ```
	/// let mut builder = NodeBuilder::default();
	///
	/// let value: AsKdl;
	/// builder.push_child_t(("name", value));
	///
	/// // No-Op: value is none and nothing can be built
	/// let value: Option<AsKdl> = None;
	/// builder.push_child_t(("name", value));
	///
	/// // No-Op: the AsKdl impl creates an empty builder, so the generated node is omitted.
	/// let empty_value: AsKdl;
	/// builder.push_child_t(("name", value, OmitIfEmpty));
	///
	/// // No-Op: the value is non-none, but the node it generates is empty, so it is omitted
	/// let empty_value: Option<AsKdl>;
	/// builder.push_child_t(("name", empty_value, OmitIfEmpty));
	/// ```
	pub fn push_child_t<'op>(&mut self, child: impl Into<NamedBuildableNode<'op>>) {
		self.push_child(child.into());
	}

	// Iterates over the provided iter to generate named nodes, omiting empty ones if the entry specifies.
	pub fn push_children<'op, V>(&mut self, iter: impl Iterator<Item = V>)
	where
		V: Into<NamedBuildableNode<'op>>,
	{
		for entry in iter {
			self.push_child_t(entry.into());
		}
	}

	// Iterates over the provided list to generate nodes with the given name, omitting empty nodes if desired.
	/// ```
	/// let mut builder = NodeBuilder::default();
	///
	/// let items: Vec<dyn AsKdl>;
	/// builder.push_children_t(("entry", items.iter()));
	///
	/// let items: Vec<dyn AsKdl>;
	/// builder.push_children_t(("entry", items.iter(), OmitIfEmpty));
	/// ```
	pub fn push_children_t<'op>(&mut self, list: impl Into<NamedBuildableNodeList<'op>>) {
		self.push_children(list.into().into_iter());
	}
}

#[derive(Clone, Copy)]
pub struct OmitIfEmpty;

pub struct BuiltNode {
	node: Option<kdl::KdlNode>,
	omit_if_empty: Option<OmitIfEmpty>,
}

impl Into<Option<kdl::KdlNode>> for BuiltNode {
	fn into(self) -> Option<kdl::KdlNode> {
		let Some(node) = self.node else {
			return None;
		};
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

impl<'builder> From<NamedBuildableNode<'builder>> for BuiltNode {
	fn from(named: NamedBuildableNode<'builder>) -> Self {
		let node = named.value.map(|value| value.as_kdl().build(named.name));
		Self {
			node,
			omit_if_empty: named.omit_if_empty,
		}
	}
}

pub struct NamedBuildableNode<'builder> {
	name: kdl::KdlIdentifier,
	value: Option<&'builder dyn AsKdl>,
	omit_if_empty: Option<OmitIfEmpty>,
}

// Base from impl, takes the name, an optional AsKdl, and a flag indicating if the generated node should be omitted if its empty
impl<'builder, K, V> From<(K, Option<&'builder V>, Option<OmitIfEmpty>)> for NamedBuildableNode<'builder>
where
	K: Into<kdl::KdlIdentifier>,
	V: AsKdl,
{
	fn from((key, value, omit_if_empty): (K, Option<&'builder V>, Option<OmitIfEmpty>)) -> Self {
		Self {
			name: key.into(),
			value: value.map(|v| v as &dyn AsKdl),
			omit_if_empty,
		}
	}
}

impl<'builder, K, V> From<(K, &'builder V)> for NamedBuildableNode<'builder>
where
	K: Into<kdl::KdlIdentifier>,
	V: AsKdl,
{
	fn from((key, value): (K, &'builder V)) -> Self {
		Self::from((key, Some(value), None))
	}
}

impl<'builder, K, V> From<(K, &'builder V, OmitIfEmpty)> for NamedBuildableNode<'builder>
where
	K: Into<kdl::KdlIdentifier>,
	V: AsKdl,
{
	fn from((key, value, omit_if_empty): (K, &'builder V, OmitIfEmpty)) -> Self {
		Self::from((key, Some(value), Some(omit_if_empty)))
	}
}

impl<'builder, K, V> From<(K, Option<&'builder V>)> for NamedBuildableNode<'builder>
where
	K: Into<kdl::KdlIdentifier>,
	V: AsKdl,
{
	fn from((key, value): (K, Option<&'builder V>)) -> Self {
		Self::from((key, value, None))
	}
}

impl<'builder, K, V> From<(K, Option<&'builder V>, OmitIfEmpty)> for NamedBuildableNode<'builder>
where
	K: Into<kdl::KdlIdentifier>,
	V: AsKdl,
{
	fn from((key, value, omit_if_empty): (K, Option<&'builder V>, OmitIfEmpty)) -> Self {
		Self::from((key, value, Some(omit_if_empty)))
	}
}

impl<'builder, K, V> From<(K, &'builder Option<V>)> for NamedBuildableNode<'builder>
where
	K: Into<kdl::KdlIdentifier>,
	V: AsKdl,
{
	fn from((key, value): (K, &'builder Option<V>)) -> Self {
		Self::from((key, value.as_ref()))
	}
}

impl<'builder, K, V> From<(K, &'builder Option<V>, OmitIfEmpty)> for NamedBuildableNode<'builder>
where
	K: Into<kdl::KdlIdentifier>,
	V: AsKdl,
{
	fn from((key, value, omit_if_empty): (K, &'builder Option<V>, OmitIfEmpty)) -> Self {
		Self::from((key, value.as_ref(), Some(omit_if_empty)))
	}
}

pub struct NamedBuildableNodeList<'op>(Vec<NamedBuildableNode<'op>>);

impl<'op> NamedBuildableNodeList<'op> {
	fn into_iter(self) -> impl Iterator<Item = NamedBuildableNode<'op>> {
		self.0.into_iter()
	}
}

impl<'op, K, I, V> From<(K, I, Option<OmitIfEmpty>)> for NamedBuildableNodeList<'op>
where
	K: Into<kdl::KdlIdentifier>,
	I: Iterator<Item = V>,
	V: AsKdl + 'op,
	NamedBuildableNode<'op>: From<(kdl::KdlIdentifier, Option<I::Item>, Option<OmitIfEmpty>)>,
{
	fn from((name, iter, omit_if_empty): (K, I, Option<OmitIfEmpty>)) -> Self {
		let name = name.into();
		let iter = iter.map(move |v| NamedBuildableNode::from((name.clone(), Some(v), omit_if_empty)));
		Self(iter.collect())
	}
}

impl<'op, K, I, V> From<(K, I)> for NamedBuildableNodeList<'op>
where
	K: Into<kdl::KdlIdentifier>,
	I: Iterator<Item = V>,
	V: AsKdl + 'op,
	NamedBuildableNode<'op>: From<(kdl::KdlIdentifier, Option<I::Item>, Option<OmitIfEmpty>)>,
{
	fn from((name, iter): (K, I)) -> Self {
		Self::from((name, iter, None))
	}
}

impl<'op, K, I, V> From<(K, I, OmitIfEmpty)> for NamedBuildableNodeList<'op>
where
	K: Into<kdl::KdlIdentifier>,
	I: Iterator<Item = V>,
	V: AsKdl + 'op,
	NamedBuildableNode<'op>: From<(kdl::KdlIdentifier, Option<I::Item>, Option<OmitIfEmpty>)>,
{
	fn from((name, iter, omit_if_empty): (K, I, OmitIfEmpty)) -> Self {
		Self::from((name, iter, Some(omit_if_empty)))
	}
}
