use locspan::Meta;
use locspan_derive::{
	StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd,
};

/// Optional value with metadata.
#[derive(
	Clone, Debug, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd, StrippedHash,
)]
#[stripped_ignore(M)]
pub struct MetaOption<T, M> {
	value: Option<Meta<T, M>>,
}

impl<T, M> From<Option<Meta<T, M>>> for MetaOption<T, M> {
	fn from(value: Option<Meta<T, M>>) -> Self {
		Self { value }
	}
}

impl<T, M> Default for MetaOption<T, M> {
	fn default() -> Self {
		Self { value: None }
	}
}

impl<T, M> MetaOption<T, M> {
	pub fn new(value: T, metadata: M) -> Self {
		Self {
			value: Some(Meta::new(value, metadata)),
		}
	}

	pub fn is_some(&self) -> bool {
		self.value.is_some()
	}

	pub fn take(&mut self) -> Self {
		Self {
			value: self.value.take(),
		}
	}

	pub fn replace(&mut self, value: T, metadata: M) -> Option<Meta<T, M>> {
		self.value.replace(Meta(value, metadata))
	}

	pub fn set_once(&mut self, metadata: M, f: impl FnOnce() -> T) {
		match self.value.as_mut() {
			Some(value) => value.1 = metadata,
			None => self.value = Some(Meta(f(), metadata)),
		}
	}

	pub fn try_unify<E>(
		&mut self,
		value: T,
		metadata: M,
		unify: impl Fn(Meta<T, M>, Meta<T, M>) -> Result<Meta<T, M>, E>,
	) -> Result<(), E> {
		match self.value.take() {
			Some(current_value) => {
				let Meta(current_value, current_metadata) = current_value;
				self.value = Some(unify(
					Meta(current_value, current_metadata),
					Meta(value, metadata),
				)?)
			}
			None => self.value = Some(Meta(value, metadata)),
		}

		Ok(())
	}

	pub fn try_set<E>(
		&mut self,
		value: T,
		metadata: M,
		on_err: impl Fn(Meta<T, M>, Meta<T, M>) -> E,
	) -> Result<(), E>
	where
		T: PartialEq,
	{
		self.try_unify(value, metadata, |Meta(a, a_meta), Meta(b, b_meta)| {
			if a == b {
				Ok(Meta(a, a_meta))
			} else {
				Err(on_err(Meta(a, a_meta), Meta(b, b_meta)))
			}
		})
	}

	pub fn try_set_opt<E>(
		&mut self,
		value: MetaOption<T, M>,
		on_err: impl Fn(Meta<T, M>, Meta<T, M>) -> E,
	) -> Result<(), E>
	where
		T: PartialEq,
	{
		match value.unwrap() {
			Some(value) => {
				let Meta(value, meta) = value;
				self.try_set(value, meta, on_err)
			}
			None => Ok(()),
		}
	}

	pub fn try_set_stripped<E>(
		&mut self,
		value: T,
		metadata: M,
		on_err: impl Fn(&Meta<T, M>, Meta<T, M>) -> E,
	) -> Result<(), E>
	where
		T: locspan::StrippedPartialEq,
	{
		match &mut self.value {
			Some(current_value) => {
				if current_value.value().stripped_eq(&value) {
					Ok(())
				} else {
					Err(on_err(current_value, Meta(value, metadata)))
				}
			}
			None => {
				self.value = Some(Meta(value, metadata));
				Ok(())
			}
		}
	}

	pub fn metadata(&self) -> Option<&M> {
		self.value.as_ref().map(Meta::metadata)
	}

	pub fn value(&self) -> Option<&T> {
		self.value.as_ref().map(Meta::value)
	}

	pub fn value_mut(&mut self) -> Option<&mut T> {
		self.value.as_mut().map(Meta::value_mut)
	}

	/// Alias for `value`
	pub fn as_ref(&self) -> Option<&Meta<T, M>> {
		self.value.as_ref()
	}

	/// Alias for `value_mut`
	pub fn as_mut(&mut self) -> Option<&mut Meta<T, M>> {
		self.value.as_mut()
	}

	pub fn into_value(self) -> Option<T> {
		self.value.map(Meta::into_value)
	}

	pub fn as_deref(&self) -> Option<&T::Target>
	where
		T: std::ops::Deref,
	{
		self.value.as_ref().map(|v| v.value().deref())
	}

	pub fn unwrap(self) -> Option<Meta<T, M>> {
		self.value
	}

	pub fn unwrap_or(self, default: Meta<T, M>) -> Meta<T, M> {
		self.value.unwrap_or(default)
	}

	pub fn unwrap_or_else(self, f: impl FnOnce() -> Meta<T, M>) -> Meta<T, M> {
		self.value.unwrap_or_else(f)
	}

	pub fn unwrap_or_else_try<E>(
		self,
		f: impl FnOnce() -> Result<Meta<T, M>, E>,
	) -> Result<Meta<T, M>, E> {
		self.value.map(Ok).unwrap_or_else(f)
	}

	pub fn or(self, other: Self) -> Self {
		if self.is_some() {
			self
		} else {
			other
		}
	}

	pub fn or_else(self, other: impl FnOnce() -> Self) -> Self {
		if self.is_some() {
			self
		} else {
			other()
		}
	}

	pub fn ok_or_else<E>(self, f: impl FnOnce() -> E) -> Result<Meta<T, M>, E> {
		self.value.ok_or_else(f)
	}

	pub fn value_or_else<E>(&self, f: impl FnOnce() -> E) -> Result<&Meta<T, M>, E> {
		self.value.as_ref().ok_or_else(f)
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> MetaOption<U, M> {
		MetaOption {
			value: self.value.map(|t| t.map(f)),
		}
	}

	pub fn try_map<U, E>(self, f: impl FnOnce(T) -> Result<U, E>) -> Result<MetaOption<U, M>, E> {
		Ok(MetaOption {
			value: self.value.map(|t| t.try_map(f)).transpose()?,
		})
	}

	pub fn map_with_causes<U>(self, f: impl FnOnce(Meta<T, M>) -> Meta<U, M>) -> MetaOption<U, M> {
		MetaOption {
			value: self.value.map(f),
		}
	}

	pub fn try_map_with_causes<U, E>(
		self,
		f: impl FnOnce(Meta<T, M>) -> Result<Meta<U, M>, E>,
	) -> Result<MetaOption<U, M>, E> {
		let value = match self.value {
			Some(t) => Some(f(t)?),
			None => None,
		};

		Ok(MetaOption { value })
	}
}

impl<T, M> From<Meta<T, M>> for MetaOption<T, M> {
	fn from(t: Meta<T, M>) -> Self {
		Self::from(Some(t))
	}
}
