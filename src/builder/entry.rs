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

pub struct Value<V: AsKdlValue>(pub V);
impl<V: AsKdlValue> Into<Entry> for Value<V> {
	fn into(self) -> Entry {
		let mut builder = Entry::default();
		builder.entry.set_value(Some(self.0.as_kdl()));
		builder
	}
}

pub struct Typed<Ty: Into<kdl::KdlIdentifier>, V: AsKdlValue>(pub Ty, pub Value<V>);
impl<Ty: Into<kdl::KdlIdentifier>, V: AsKdlValue> Into<Entry> for Typed<Ty, V> {
	fn into(self) -> Entry {
		let mut builder: Entry = self.1.into();
		builder.entry.set_ty(self.0);
		builder
	}
}

pub struct Property<K: Into<kdl::KdlIdentifier>, V: Into<Entry>>(pub K, pub V);
impl<K: Into<kdl::KdlIdentifier>, V: Into<Entry>> Into<Entry> for Property<K, V> {
	fn into(self) -> Entry {
		let mut builder: Entry = self.1.into();
		builder.entry.set_name(Some(self.0));
		builder
	}
}
