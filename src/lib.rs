pub mod builder;
pub mod error;
pub mod reader;

use error::ValueTypeMismatch;

pub trait NodeId {
	fn id() -> &'static str
	where
		Self: Sized;

	fn get_id(&self) -> &'static str;
}

#[macro_export]
macro_rules! impl_kdl_node {
	($target:ty, $id:expr) => {
		impl kdlize::NodeId for $target {
			fn id() -> &'static str {
				$id
			}

			fn get_id(&self) -> &'static str {
				$id
			}
		}
	};
}

pub trait FromKdlValue<'doc> {
	type Error;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error>
	where
		Self: Sized;
}

pub trait AsKdlValue {
	fn as_kdl(&self) -> kdl::KdlValue;
}
impl<V: AsKdlValue> AsKdlValue for &V {
	fn as_kdl(&self) -> kdl::KdlValue {
		V::as_kdl(self)
	}
}
impl<V: AsKdlValue + Clone> AsKdlValue for std::borrow::Cow<'_, V> {
	fn as_kdl(&self) -> kdl::KdlValue {
		V::as_kdl(self.as_ref())
	}
}

impl<'doc> FromKdlValue<'doc> for &'doc kdl::KdlValue {
	type Error = std::convert::Infallible;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
		Ok(value)
	}
}

impl<'doc> FromKdlValue<'doc> for &'doc str {
	type Error = ValueTypeMismatch;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
		match value {
			kdl::KdlValue::String(value) => Ok(value.as_str()),
			_ => Err(ValueTypeMismatch::new(&value, "String")),
		}
	}
}
impl<'doc> FromKdlValue<'doc> for String {
	type Error = ValueTypeMismatch;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
		Ok(<&str>::from_kdl(value)?.to_owned())
	}
}
impl AsKdlValue for str {
	fn as_kdl(&self) -> kdl::KdlValue {
		kdl::KdlValue::String(self.to_owned())
	}
}
impl AsKdlValue for &str {
	fn as_kdl(&self) -> kdl::KdlValue {
		str::as_kdl(self)
	}
}
impl AsKdlValue for String {
	fn as_kdl(&self) -> kdl::KdlValue {
		self.as_str().as_kdl()
	}
}
impl<'doc> FromKdlValue<'doc> for &'doc std::path::Path {
	type Error = ValueTypeMismatch;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
		Ok(std::path::Path::new(<&str>::from_kdl(value)?))
	}
}
impl<'doc> FromKdlValue<'doc> for std::path::PathBuf {
	type Error = ValueTypeMismatch;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
		Ok(<&std::path::Path>::from_kdl(value)?.to_owned())
	}
}
impl AsKdlValue for std::path::PathBuf {
	fn as_kdl(&self) -> kdl::KdlValue {
		self.to_str().as_kdl()
	}
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum FailedToParseValueString {
	#[error(transparent)]
	#[diagnostic(transparent)]
	TypeMismatch(crate::error::ValueTypeMismatch),
	#[error(transparent)]
	#[diagnostic(transparent)]
	FromStr(Box<dyn miette::Diagnostic + Send + Sync>),
}

// Implements FromKdlValue and AsKdlValue for the provided type,
// such that the expected value is a string, and the provided type is parsed to/from a string using FromStr/ToString.
#[macro_export]
macro_rules! impl_kdlvalue_str {
	($target:ty) => {
		impl<'doc> $crate::FromKdlValue<'doc> for $target
		where
			$target: std::str::FromStr,
			miette::Report: From<<$target as std::str::FromStr>::Err>,
		{
			type Error = miette::Report;
			fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
				let value = match value {
					kdl::KdlValue::String(value) => value,
					_ => {
						let type_mismatch = $crate::error::ValueTypeMismatch::new(&value, "String");
						return Err(miette::Report::new(type_mismatch))
					}
				};
				let result = <$target as std::str::FromStr>::from_str(value);
				//let result = result.map_err(|err| $crate::FailedToParseValueString::FromStr(Box::new(err)));
				Ok(result?)
			}
		}
		impl $crate::AsKdlValue for $target
		where
			$target: ToString,
		{
			fn as_kdl(&self) -> kdl::KdlValue {
				let s = self.to_string();
				match s.is_empty() {
					true => kdl::KdlValue::Null,
					false => kdl::KdlValue::String(s),
				}
			}
		}
	};
}

#[cfg(test)]
mod test {
	struct ExampleStr;
	impl std::str::FromStr for ExampleStr {
		type Err = std::convert::Infallible;
		fn from_str(_s: &str) -> Result<Self, Self::Err> {
			Ok(Self)
		}
	}
	impl std::fmt::Display for ExampleStr {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "Example")
		}
	}
	impl_kdlvalue_str!(ExampleStr);
}

macro_rules! impl_kdlvalue_primitive {
	($target:ty, $actual:ty) => {
		impl<'doc> FromKdlValue<'doc> for $target {
			type Error = ValueTypeMismatch;
			fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
				Ok(<$actual>::from_kdl(value)? as $target)
			}
		}
		impl AsKdlValue for $target {
			fn as_kdl(&self) -> kdl::KdlValue {
				(*self as $actual).as_kdl()
			}
		}
	};
}

impl<'doc> FromKdlValue<'doc> for i128 {
	type Error = ValueTypeMismatch;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
		match value {
			kdl::KdlValue::Integer(value) => Ok(*value),
			_ => Err(ValueTypeMismatch::new(&value, "Integer")),
		}
	}
}
impl AsKdlValue for i128 {
	fn as_kdl(&self) -> kdl::KdlValue {
		kdl::KdlValue::Integer(*self)
	}
}
impl_kdlvalue_primitive!(u8, i128);
impl_kdlvalue_primitive!(i8, i128);
impl_kdlvalue_primitive!(u16, i128);
impl_kdlvalue_primitive!(i16, i128);
impl_kdlvalue_primitive!(u32, i128);
impl_kdlvalue_primitive!(i32, i128);
impl_kdlvalue_primitive!(u64, i128);
impl_kdlvalue_primitive!(i64, i128);
impl_kdlvalue_primitive!(u128, i128);
//impl_kdlvalue_primitive!(i128,  i128);
impl_kdlvalue_primitive!(usize, i128);
impl_kdlvalue_primitive!(isize, i128);

impl<'doc> FromKdlValue<'doc> for f64 {
	type Error = ValueTypeMismatch;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
		match value {
			kdl::KdlValue::Float(value) => Ok(*value),
			_ => Err(ValueTypeMismatch::new(&value, "Float")),
		}
	}
}
impl AsKdlValue for f64 {
	fn as_kdl(&self) -> kdl::KdlValue {
		kdl::KdlValue::Float(*self)
	}
}
impl_kdlvalue_primitive!(f32, f64);
//impl_kdlvalue_primitive!(f64,   f64);

impl<'doc> FromKdlValue<'doc> for bool {
	type Error = ValueTypeMismatch;
	fn from_kdl(value: &'doc kdl::KdlValue) -> Result<Self, Self::Error> {
		match value {
			kdl::KdlValue::Bool(value) => Ok(*value),
			_ => Err(ValueTypeMismatch::new(&value, "Bool")),
		}
	}
}
impl AsKdlValue for bool {
	fn as_kdl(&self) -> kdl::KdlValue {
		kdl::KdlValue::Bool(*self)
	}
}

impl<V> AsKdlValue for Option<V>
where
	V: AsKdlValue,
{
	fn as_kdl(&self) -> kdl::KdlValue {
		match self {
			None => kdl::KdlValue::Null,
			Some(value) => value.as_kdl(),
		}
	}
}

pub trait FromKdlNode<'doc, Context> {
	type Error;
	fn from_kdl(node: &mut reader::Node<'doc, Context>) -> Result<Self, Self::Error>
	where
		Self: Sized;
}
pub trait AsKdlNode {
	fn as_kdl(&self) -> builder::Node;
}
impl<V: AsKdlNode> AsKdlNode for &V {
	fn as_kdl(&self) -> builder::Node {
		(*self).as_kdl()
	}
}
impl<V: AsKdlNode> AsKdlNode for Option<V> {
	fn as_kdl(&self) -> builder::Node {
		match self {
			None => builder::Node::default(),
			Some(value) => value.as_kdl(),
		}
	}
}

pub trait DocumentExt {
	fn to_string_unescaped(&self) -> String;
}
impl DocumentExt for kdl::KdlDocument {
	fn to_string_unescaped(&self) -> String {
		let doc = self.to_string();
		let doc = doc.replace("\\r", "");
		let doc = doc.replace("\\n", "\n");
		let doc = doc.replace("\\t", "\t");
		let doc = doc.replace("    ", "\t");
		doc
	}
}
