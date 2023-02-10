use super::{
	non_functional::PropertyValues, required_functional, PropertyValue, PropertyValueRef,
	RequiredFunctionalPropertyValue,
};

/// Functional property value.
#[derive(Debug, Clone)]
pub struct FunctionalPropertyValue<T, M>(Option<RequiredFunctionalPropertyValue<T, M>>);

impl<T, M> Default for FunctionalPropertyValue<T, M> {
	fn default() -> Self {
		Self(None)
	}
}

impl<T, M> FunctionalPropertyValue<T, M> {
	pub fn new(
		value: Option<RequiredFunctionalPropertyValue<T, M>>
	) -> Self {
		Self(value)
	}

	pub fn as_required(&self) -> Option<&RequiredFunctionalPropertyValue<T, M>> {
		self.0.as_ref()
	}

	pub fn into_required(self) -> Option<RequiredFunctionalPropertyValue<T, M>> {
		self.0
	}

	pub fn sub_properties(&self) -> Option<&PropertyValues<(), M>> {
		self.0
			.as_ref()
			.map(RequiredFunctionalPropertyValue::sub_properties)
	}

	pub fn sub_property_metadata(&self) -> Option<&M> {
		self.0
			.as_ref()
			.map(RequiredFunctionalPropertyValue::sub_property_metadata)
	}

	pub fn value(&self) -> Option<&T> {
		self.0.as_ref().map(RequiredFunctionalPropertyValue::value)
	}

	pub fn is_some(&self) -> bool {
		self.value().is_some()
	}

	pub fn is_some_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
		self.value().map(f).unwrap_or_default()
	}

	pub fn is_none(&self) -> bool {
		self.value().is_none()
	}

	pub fn iter(&self) -> Iter<T, M> {
		match self.0.as_ref() {
			Some(i) => Iter::Some(i.iter()),
			None => Iter::None,
		}
	}

	pub fn try_map_borrow_metadata<U, E>(
		self,
		f: impl FnOnce(T, &PropertyValues<(), M>) -> Result<U, E>
	) -> Result<FunctionalPropertyValue<U, M>, E> {
		match self.0 {
			Some(inner) => Ok(FunctionalPropertyValue(Some(inner.try_map_borrow_metadata(f)?))),
			None => Ok(FunctionalPropertyValue(None))
		}
	}
}

pub enum Iter<'a, T, M> {
	None,
	Some(required_functional::Iter<'a, T, M>),
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
			Self::Some(i) => i.size_hint(),
			Self::None => (0, Some(0)),
		}
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Some(i) => i.next(),
			Self::None => None,
		}
	}
}

impl<'a, T, M> ExactSizeIterator for Iter<'a, T, M> {}

impl<'a, T, M> DoubleEndedIterator for Iter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::Some(i) => i.next_back(),
			Self::None => None,
		}
	}
}

pub enum IntoIter<T, M> {
	Some(required_functional::IntoIter<T, M>),
	None,
}

impl<T: Clone, M> Iterator for IntoIter<T, M> {
	type Item = PropertyValue<T, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Self::Some(i) => i.size_hint(),
			Self::None => (0, Some(0)),
		}
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Some(i) => i.next_back(),
			Self::None => None,
		}
	}
}

impl<T: Clone, M> ExactSizeIterator for IntoIter<T, M> {}

impl<T: Clone, M> DoubleEndedIterator for IntoIter<T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::Some(i) => i.next_back(),
			Self::None => None,
		}
	}
}
