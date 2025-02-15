use super::Typed;
use crate::AsKdlValue;

#[derive(Debug)]
pub struct Entry {
	pub(super) entry: kdl::KdlEntry,
}

impl Default for Entry {
	fn default() -> Self {
		Self {
			entry: kdl::KdlEntry::new(kdl::KdlValue::Null),
		}
	}
}

impl Entry {
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
}

pub trait InnerValue {
	type Inner;
	fn inner(&self) -> &Self::Inner;
}

pub struct Value<V>(pub V);
impl<V> InnerValue for Value<&V> {
	type Inner = V;
	fn inner(&self) -> &Self::Inner {
		self.0
	}
}
impl<V: AsKdlValue> Into<Entry> for Value<V> {
	fn into(self) -> Entry {
		let mut builder = Entry::default();
		builder.entry.set_value(Some(self.0.as_kdl()));
		builder
	}
}

impl<Ty: Into<kdl::KdlIdentifier>, I, V> InnerValue for Typed<Ty, I>
where
	I: InnerValue<Inner = V>,
{
	type Inner = V;
	fn inner(&self) -> &Self::Inner {
		self.1.inner()
	}
}
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlValue> Into<Entry> for Typed<Ty, Value<V>> {
	fn into(self) -> Entry {
		let mut builder: Entry = self.1.into();
		builder.entry.set_ty(self.0);
		builder
	}
}

pub struct Property<K: Into<kdl::KdlIdentifier>, V: Into<Entry>>(pub K, pub V);
impl<K: Into<kdl::KdlIdentifier>, I, V> InnerValue for Property<K, I>
where
	I: InnerValue<Inner = V> + Into<Entry>,
{
	type Inner = V;
	fn inner(&self) -> &Self::Inner {
		self.1.inner()
	}
}
impl<K: Into<kdl::KdlIdentifier>, V: Into<Entry>> Into<Entry> for Property<K, V> {
	fn into(self) -> Entry {
		let mut builder: Entry = self.1.into();
		builder.entry.set_name(Some(self.0));
		builder
	}
}
