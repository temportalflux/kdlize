#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
	#[error(transparent)]
	InvalidQuery(#[from] InvalidQueryFormat),
	#[error(transparent)]
	MissingAtQuery(#[from] QueryMissing),
	#[error(transparent)]
	InvalidValueType(#[from] InvalidValueType),
	#[error(transparent)]
	MissingEntry(#[from] MissingEntryValue),
	#[error(transparent)]
	MissingEntryType(#[from] MissingEntryType),
	#[error(transparent)]
	MissingChildren(#[from] NoChildren),
	#[error(transparent)]
	MissingDocChildren(#[from] EmptyDocument),
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("Entry \"{0}\" is missing a type identifier")]
pub struct MissingEntryType(pub kdl::KdlEntry);

/// The kdl value did not match the expected type.
#[derive(thiserror::Error, Debug, Clone)]
#[error("Invalid value {0:?}, was expecting a {1}")]
pub struct InvalidValueType(pub kdl::KdlValue, pub &'static str);

/// The node is missing an entry that was required.
#[derive(thiserror::Error, Debug, Clone)]
pub struct MissingEntryValue(pub kdl::KdlNode, pub kdl::NodeKey);
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
#[error("Query for {1:?} does not exist in {0}")]
pub struct QueryMissing(pub kdl::KdlDocument, pub String);

#[derive(thiserror::Error, Debug, Clone)]
#[error(transparent)]
pub struct InvalidQueryFormat(#[from] pub kdl::KdlError);

#[derive(thiserror::Error, Debug, Clone)]
#[error("Expected node to have children, but none are present in {0}")]
pub struct NoChildren(pub kdl::KdlNode);

#[derive(thiserror::Error, Debug, Clone)]
#[error("Document has no children")]
pub struct EmptyDocument(pub kdl::KdlDocument);
