
/// The node is missing an entry that was required.
#[derive(thiserror::Error, Debug, Clone)]
pub struct MissingEntryValue(kdl::KdlNode, kdl::NodeKey);
impl MissingEntryValue {
	pub(crate) fn new_index(node: kdl::KdlNode, idx: usize) -> Self {
		Self(node, kdl::NodeKey::Index(idx))
	}
	pub(crate) fn new_prop(node: kdl::KdlNode, key: impl AsRef<str>) -> Self {
		Self(node, kdl::NodeKey::Key(kdl::KdlIdentifier::from(key.as_ref())))
	}
}
impl std::fmt::Display for MissingEntryValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.1 {
			kdl::NodeKey::Index(v) => write!(f, "Node {} is missing an entry at index {v}", self.0),
			kdl::NodeKey::Key(v) => {
				write!(f, "Node {} is missing an entry at property {}", self.0, v.value())
			}
		}
	}
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("Entry \"{0}\" is missing a type identifier")]
pub struct MissingEntryType(pub(crate) kdl::KdlEntry);

#[derive(thiserror::Error, Debug, Clone)]
#[error("Node {0} is missing a child node with name \"{1}\"")]
pub struct MissingChild(pub(crate) kdl::KdlNode, pub(crate) &'static str);

#[derive(thiserror::Error, Debug)]
pub enum MissingTypedEntry {
	#[error(transparent)]
	MissingValue(#[from] MissingEntryValue),
	#[error(transparent)]
	MissingType(#[from] MissingEntryType),
}

#[derive(thiserror::Error, Debug)]
pub enum RequiredValue<E> {
	#[error(transparent)]
	Missing(MissingEntryValue),
	#[error(transparent)]
	Parse(#[from] E),
}

#[derive(thiserror::Error, Debug)]
pub enum RequiredChild<E> {
	#[error(transparent)]
	Missing(MissingChild),
	#[error(transparent)]
	Parse(#[from] E),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseValueFromStr<TError> {
	// Could not parse the kdl value as an str
	#[error(transparent)]
	FailedToParse(#[from] ValueTypeMismatch),
	// Could not convert str to T
	#[error(transparent)]
	FailedToInterpret(TError),
}

#[derive(thiserror::Error, Debug)]
#[error("Expected '{2}' to be a '{0}', but it is a '{1}'.")]
pub struct ValueTypeMismatch(&'static str, &'static str, kdl::KdlValue);
impl ValueTypeMismatch {
	pub(crate) fn new(value: &kdl::KdlValue, desired: &'static str) -> Self {
		let actual_name = match value {
			kdl::KdlValue::String(_) => "String",
			kdl::KdlValue::Integer(_) => "Integer",
			kdl::KdlValue::Float(_) => "Float",
			kdl::KdlValue::Bool(_) => "Bool",
			kdl::KdlValue::Null => "Null",
		};
		Self(desired, actual_name, value.clone())
	}
}
