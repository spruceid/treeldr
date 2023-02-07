use std::ops::{Deref, DerefMut};

use locspan::Meta;

use super::{non_functional, PropertyValue, PropertyValueRef, PropertyValues};

/// Required functional property value.
#[derive(Debug, Clone)]
pub struct RequiredFunctionalPropertyValue<T, M> {
	sub_properties: PropertyValues<(), M>,
	value: T,
}

impl<T, M> RequiredFunctionalPropertyValue<T, M> {
	pub fn new(sub_properties: PropertyValues<(), M>, value: T) -> Self {
		assert!(!sub_properties.is_empty());
		Self {
			sub_properties,
			value,
		}
	}

	pub fn sub_properties(&self) -> &PropertyValues<(), M> {
		&self.sub_properties
	}

	pub fn sub_property_metadata(&self) -> &M {
		self.sub_properties.first().unwrap().value.metadata()
	}

	pub fn value(&self) -> &T {
		&self.value
	}

	pub fn value_mut(&mut self) -> &mut T {
		&mut self.value
	}

	pub fn iter(&self) -> Iter<T, M> {
		Iter {
			sub_properties: self.sub_properties.iter(),
			value: &self.value,
		}
	}

	pub fn try_map_borrow_metadata<U, E>(self, f: impl FnOnce(T, &PropertyValues<(), M>) -> Result<U, E>) -> Result<RequiredFunctionalPropertyValue<U, M>, E> {
		let value = f(self.value, &self.sub_properties)?;
		Ok(RequiredFunctionalPropertyValue {
			sub_properties: self.sub_properties,
			value
		})
	}
}

impl<T, M> Deref for RequiredFunctionalPropertyValue<T, M> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T, M> DerefMut for RequiredFunctionalPropertyValue<T, M> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<'a, T, M> IntoIterator for &'a RequiredFunctionalPropertyValue<T, M> {
	type IntoIter = Iter<'a, T, M>;
	type Item = PropertyValueRef<'a, T, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<T: Clone, M> IntoIterator for RequiredFunctionalPropertyValue<T, M> {
	type IntoIter = IntoIter<T, M>;
	type Item = PropertyValue<T, M>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			sub_properties: self.sub_properties.into_iter(),
			value: self.value,
		}
	}
}

pub struct Iter<'a, T, M> {
	sub_properties: non_functional::Iter<'a, (), M>,
	value: &'a T,
}

impl<'a, T, M> Iterator for Iter<'a, T, M> {
	type Item = PropertyValueRef<'a, T, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.sub_properties.size_hint()
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.sub_properties.next().map(|s| {
			PropertyValueRef::new(s.sub_property, Meta(self.value, s.value.into_metadata()))
		})
	}
}

impl<'a, T, M> ExactSizeIterator for Iter<'a, T, M> {}

impl<'a, T, M> DoubleEndedIterator for Iter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.sub_properties.next_back().map(|s| {
			PropertyValueRef::new(s.sub_property, Meta(self.value, s.value.into_metadata()))
		})
	}
}

pub struct IntoIter<T, M> {
	sub_properties: non_functional::IntoIter<(), M>,
	value: T,
}

impl<T: Clone, M> Iterator for IntoIter<T, M> {
	type Item = PropertyValue<T, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.sub_properties.size_hint()
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.sub_properties.next().map(|s| {
			PropertyValue::new(
				s.sub_property,
				Meta(self.value.clone(), s.value.into_metadata()),
			)
		})
	}
}

impl<T: Clone, M> ExactSizeIterator for IntoIter<T, M> {}

impl<T: Clone, M> DoubleEndedIterator for IntoIter<T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.sub_properties.next_back().map(|s| {
			PropertyValue::new(
				s.sub_property,
				Meta(self.value.clone(), s.value.into_metadata()),
			)
		})
	}
}
