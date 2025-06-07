#[derive(thiserror::Error, Debug, PartialEq)]
pub enum QueryError {
	#[error(transparent)]
	MissingValue(#[from] MissingEntry),
	#[error(transparent)]
	MissingType(#[from] MissingEntryType),
	#[error(transparent)]
	ValueTypeMismatch(#[from] ValueTypeMismatch),
	#[error(transparent)]
	MissingChild(#[from] NodeMissingChild),
}
impl From<RequiredValue<ValueTypeMismatch>> for QueryError {
	fn from(value: RequiredValue<ValueTypeMismatch>) -> Self {
		match value {
			RequiredValue::Missing(missing) => Self::MissingValue(missing),
			RequiredValue::Parse(mismatch) => Self::ValueTypeMismatch(mismatch),
		}
	}
}

/// The node is missing an entry that was required.
#[derive(thiserror::Error, Debug, Clone, PartialEq, miette::Diagnostic)]
#[diagnostic(code(kdlize::missing_entry))]
pub struct MissingEntry {
	#[source_code]
	pub(crate) src: String,
	pub(crate) span: miette::LabeledSpan,
	pub(crate) key: kdl::NodeKey,
}
impl MissingEntry {
	pub(crate) fn new_index(node: kdl::KdlNode, idx: usize) -> Self {
		let src = node.to_string();
		let label = format!("missing value at index {idx}");
		let span = miette::LabeledSpan::new_primary_with_span(Some(label), (0, src.len()));
		Self {
			src,
			span,
			key: kdl::NodeKey::Index(idx),
		}
	}
	pub(crate) fn new_prop(node: kdl::KdlNode, key: impl AsRef<str>) -> Self {
		let src = node.to_string();
		let label = format!("missing value at property {:?}", key.as_ref());
		let span = miette::LabeledSpan::new_primary_with_span(Some(label), (0, src.len()));
		Self {
			src,
			span,
			key: kdl::NodeKey::Key(kdl::KdlIdentifier::from(key.as_ref())),
		}
	}
}
impl std::fmt::Display for MissingEntry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.key {
			kdl::NodeKey::Index(v) => write!(f, "Node {:?} is missing an entry at index {v}", self.src),
			kdl::NodeKey::Key(v) => {
				write!(f, "Node {:?} is missing an entry at property {}", self.src, v.value())
			}
		}
	}
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, miette::Diagnostic)]
#[error("Entry {value:?} is missing a type identifier")]
#[diagnostic(code(kdlize::entry_missing_type))]
pub struct MissingEntryType {
	#[source_code]
	pub(crate) src: String,
	#[label("missing type annotation")]
	pub(crate) span: miette::SourceSpan,
	pub(crate) value: kdl::KdlEntry,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, miette::Diagnostic)]
#[error("Node {src:?} is missing a child node with name \"{child_name}\"")]
#[diagnostic(code(kdlize::node_missing_child))]
pub struct NodeMissingChild {
	#[source_code]
	pub src: String,
	pub span: miette::LabeledSpan,
	pub child_name: kdl::KdlIdentifier,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, miette::Diagnostic)]
#[error("Node is missing children document")]
#[diagnostic(code(kdlize::missing_node_document))]
pub struct MissingNodeDocument {
	#[source_code]
	pub(crate) src: String,
	#[label("missing children")]
	pub(crate) node_span: miette::SourceSpan,
}

#[derive(thiserror::Error, Debug)]
pub enum MissingTypedEntry {
	#[error(transparent)]
	MissingValue(#[from] MissingEntry),
	#[error(transparent)]
	MissingType(#[from] MissingEntryType),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum RequiredValue<E> {
	#[error(transparent)]
	Missing(MissingEntry),
	#[error(transparent)]
	Parse(E),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum RequiredChild<E> {
	#[error(transparent)]
	Missing(NodeMissingChild),
	#[error(transparent)]
	Parse(#[from] E),
}

#[derive(thiserror::Error, Debug, PartialEq, miette::Diagnostic)]
#[error("Expected '{value}' to be a '{expected_type}', but it is a '{actual_type}'.")]
#[diagnostic(code(kdlize::value_unexpected_type))]
pub struct ValueTypeMismatch {
	pub expected_type: &'static str,
	pub actual_type: &'static str,
	pub value: kdl::KdlValue,
}
impl ValueTypeMismatch {
	pub fn new(value: &kdl::KdlValue, desired: &'static str) -> Self {
		let actual_name = match value {
			kdl::KdlValue::String(_) => "String",
			kdl::KdlValue::Integer(_) => "Integer",
			kdl::KdlValue::Float(_) => "Float",
			kdl::KdlValue::Bool(_) => "Bool",
			kdl::KdlValue::Null => "Null",
		};
		Self {
			expected_type: desired,
			actual_type: actual_name,
			value: value.clone(),
		}
	}
}
