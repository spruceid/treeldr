use crate::Id;
use locspan::Meta;
use std::ops::Deref;

pub mod functional;
pub mod non_functional;
pub mod required_functional;

pub use functional::FunctionalPropertyValue;
pub use non_functional::PropertyValues;
pub use required_functional::RequiredFunctionalPropertyValue;

pub enum SubPropertyIn<P> {
	Known(P),
	Other(Id),
}

/// Property value.
#[derive(Clone, Debug)]
pub struct PropertyValue<T, M> {
	/// Sub-property this value is associated to.
	pub sub_property: Option<Id>,

	/// Value itself.
	pub value: Meta<T, M>,
}

impl<T, M> PropertyValue<T, M> {
	pub fn new(sub_property: Option<Id>, value: Meta<T, M>) -> Self {
		Self {
			sub_property,
			value,
		}
	}

	fn for_sub_property(self, id: Id) -> Self {
		Self {
			sub_property: self.sub_property.or(Some(id)),
			value: self.value,
		}
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> PropertyValue<U, M> {
		PropertyValue { sub_property: self.sub_property, value: self.value.map(f) }
	}

	pub(crate) fn into_class_binding<B>(self, binding: impl Fn(Option<Id>, T) -> B) -> Meta<B, M> {
		let Meta(v, meta) = self.value;
		Meta(binding(self.sub_property, v), meta)
	}
}

impl<'a, T, M> PropertyValue<T, &'a M> {
	pub fn into_cloned_metadata(self) -> PropertyValue<T, M> where M: Clone {
		PropertyValue::new(self.sub_property, self.value.into_cloned_metadata())
	}
}

/// Reference to a property value.
pub struct PropertyValueRef<'a, T, M> {
	/// Sub-property this value is associated to.
	pub sub_property: Option<Id>,

	/// Value itself.
	pub value: Meta<&'a T, &'a M>,
}

impl<'a, T, M> PropertyValueRef<'a, T, M> {
	pub fn new(sub_property: Option<Id>, value: Meta<&'a T, &'a M>) -> Self {
		Self {
			sub_property,
			value,
		}
	}

	fn for_sub_property(self, id: Id) -> Self {
		Self {
			sub_property: self.sub_property.or(Some(id)),
			value: self.value,
		}
	}

	pub(crate) fn into_class_binding<B>(
		self,
		binding: impl Fn(Option<Id>, &'a T) -> B,
	) -> Meta<B, &'a M> {
		let Meta(v, meta) = self.value;
		Meta(binding(self.sub_property, v), meta)
	}

	pub(crate) fn into_cloned_class_binding<B>(
		self,
		binding: impl Fn(Option<Id>, T) -> B,
	) -> Meta<B, &'a M>
	where
		T: Clone,
	{
		let Meta(v, meta) = self.value.into_cloned_value();
		Meta(binding(self.sub_property, v), meta)
	}

	pub(crate) fn into_deref_class_binding<B>(
		self,
		binding: impl Fn(Option<Id>, &'a T::Target) -> B,
	) -> Meta<B, &'a M>
	where
		T: Deref,
	{
		let v = self.value.0.deref();
		let meta = self.value.1;
		Meta(binding(self.sub_property, v), meta)
	}
}

impl<'a, T, M> Clone for PropertyValueRef<'a, T, M> {
	fn clone(&self) -> Self {
		Self {
			sub_property: self.sub_property,
			value: self.value,
		}
	}
}

impl<'a, T, M> Copy for PropertyValueRef<'a, T, M> {}

pub enum Iter<'a, T, M> {
	NonFunctional(non_functional::Iter<'a, T, M>),
	RequiredFunctional(required_functional::Iter<'a, T, M>),
	None,
}

impl<'a, T, M> Default for Iter<'a, T, M> {
	fn default() -> Self {
		Self::None
	}
}

impl<'a, T, M> Iterator for Iter<'a, T, M> {
	type Item = PropertyValueRef<'a, T, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Self::NonFunctional(i) => i.size_hint(),
			Self::RequiredFunctional(i) => i.size_hint(),
			Self::None => (0, Some(0)),
		}
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::NonFunctional(i) => i.next(),
			Self::RequiredFunctional(i) => i.next(),
			Self::None => None,
		}
	}
}

impl<'a, T, M> ExactSizeIterator for Iter<'a, T, M> {}

impl<'a, T, M> DoubleEndedIterator for Iter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::NonFunctional(i) => i.next_back(),
			Self::RequiredFunctional(i) => i.next_back(),
			Self::None => None,
		}
	}
}
