#[derive(thiserror::Error, Debug, PartialEq)]
pub enum QueryError {
	#[error(transparent)]
	MissingValue(#[from] MissingEntry),
	#[error(transparent)]
	MissingType(#[from] MissingEntryType),
	#[error(transparent)]
	ValueTypeMismatch(#[from] ValueTypeMismatch),
	#[error(transparent)]
	MissingChild(#[from] MissingChild),
}
impl From<RequiredValue<ValueTypeMismatch>> for QueryError {
	fn from(value: RequiredValue<ValueTypeMismatch>) -> Self {
		match value {
			RequiredValue::Missing(missing) => Self::MissingValue(missing),
			RequiredValue::Parse(mismatch) => Self::ValueTypeMismatch(mismatch),
		}
	}
}
impl From<ParseValueFromStr<std::convert::Infallible>> for QueryError {
	fn from(value: ParseValueFromStr<std::convert::Infallible>) -> Self {
		match value {
			ParseValueFromStr::FailedToParse(mismatch) => Self::ValueTypeMismatch(mismatch),
			ParseValueFromStr::FailedToInterpret(_infallible) => panic!("infallible parse from str"),
		}
	}
}

/// The node is missing an entry that was required.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub struct MissingEntry(kdl::KdlNode, kdl::NodeKey);
impl MissingEntry {
	pub(crate) fn new_index(node: kdl::KdlNode, idx: usize) -> Self {
		Self(node, kdl::NodeKey::Index(idx))
	}
	pub(crate) fn new_prop(node: kdl::KdlNode, key: impl AsRef<str>) -> Self {
		Self(node, kdl::NodeKey::Key(kdl::KdlIdentifier::from(key.as_ref())))
	}
}
impl std::fmt::Display for MissingEntry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.1 {
			kdl::NodeKey::Index(v) => write!(f, "Node {} is missing an entry at index {v}", self.0),
			kdl::NodeKey::Key(v) => {
				write!(f, "Node {} is missing an entry at property {}", self.0, v.value())
			}
		}
	}
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("Entry \"{0}\" is missing a type identifier")]
pub struct MissingEntryType(pub(crate) kdl::KdlEntry);

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("Node {0} is missing a child node with name \"{1}\"")]
pub struct MissingChild(pub(crate) kdl::KdlNode, pub(crate) kdl::KdlIdentifier);

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("Node \"{0}\" is missing children/subdocument")]
pub struct MissingDocument(pub(crate) kdl::KdlNode);

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

#[derive(thiserror::Error, Debug, PartialEq)]
#[error("Expected '{2}' to be a '{0}', but it is a '{1}'.")]
pub struct ValueTypeMismatch(&'static str, &'static str, kdl::KdlValue);
impl ValueTypeMismatch {
	pub fn new(value: &kdl::KdlValue, desired: &'static str) -> Self {
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
