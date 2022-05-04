use crate::{Caused, Causes, WithCauses};
use locspan::Location;

#[derive(Clone, Debug)]
pub struct MaybeSet<T, F> {
	value: Option<WithCauses<T, F>>,
}

impl<T, F> From<Option<WithCauses<T, F>>> for MaybeSet<T, F> {
	fn from(value: Option<WithCauses<T, F>>) -> Self {
		Self { value }
	}
}

impl<T, F> Default for MaybeSet<T, F> {
	fn default() -> Self {
		Self { value: None }
	}
}

impl<T, F> MaybeSet<T, F> {
	pub fn new(value: T, causes: impl Into<Causes<F>>) -> Self {
		Self {
			value: Some(WithCauses::new(value, causes)),
		}
	}

	pub fn is_set(&self) -> bool {
		self.value.is_some()
	}

	pub fn take(&mut self) -> Self {
		Self {
			value: self.value.take(),
		}
	}

	pub fn replace(&mut self, value: T, cause: Option<Location<F>>) -> Option<WithCauses<T, F>>
	where
		F: Ord,
	{
		self.value.replace(WithCauses::new(value, cause))
	}

	pub fn set_once(&mut self, cause: Option<Location<F>>, f: impl FnOnce() -> T)
	where
		F: Ord,
	{
		match self.value.as_mut() {
			Some(value) => value.add_opt_cause(cause),
			None => self.value = Some(WithCauses::new(f(), cause)),
		}
	}

	pub fn try_unify<E>(
		&mut self,
		value: T,
		causes: impl Into<Causes<F>>,
		unify: impl Fn(T, T, &Causes<F>, &Causes<F>) -> Result<T, E>,
	) -> Result<(), E>
	where
		F: Ord,
	{
		match self.value.take() {
			Some(current_value) => {
				let (current_value, current_causes) = current_value.into_parts();
				let causes = causes.into();
				self.value = Some(WithCauses::new(
					unify(current_value, value, &current_causes, &causes)?,
					current_causes.with(causes),
				))
			}
			None => self.value = Some(WithCauses::new(value, causes)),
		}

		Ok(())
	}

	pub fn try_set<E>(
		&mut self,
		value: T,
		causes: impl Into<Causes<F>>,
		on_err: impl Fn(T, T, &Causes<F>, &Causes<F>) -> E,
	) -> Result<(), E>
	where
		T: PartialEq,
		F: Ord,
	{
		self.try_unify(value, causes, |a, b, a_causes, b_causes| {
			if a == b {
				Ok(a)
			} else {
				Err(on_err(a, b, a_causes, b_causes))
			}
		})
	}

	pub fn try_set_opt<E>(
		&mut self,
		value: MaybeSet<T, F>,
		on_err: impl Fn(T, T, &Causes<F>, &Causes<F>) -> E,
	) -> Result<(), E>
	where
		T: PartialEq,
		F: Ord,
	{
		match value.unwrap() {
			Some(value) => {
				let (value, causes) = value.into_parts();
				self.try_set(value, causes, on_err)
			}
			None => Ok(()),
		}
	}

	pub fn try_set_stripped<E>(
		&mut self,
		value: T,
		cause: Option<Location<F>>,
		on_err: impl Fn(&T, Option<&Location<F>>, T) -> E,
	) -> Result<(), Caused<E, F>>
	where
		T: locspan::StrippedPartialEq,
		F: Ord,
	{
		match &mut self.value {
			Some(current_value) => {
				if current_value.inner().stripped_eq(&value) {
					current_value.add_opt_cause(cause);
					Ok(())
				} else {
					Err(Caused::new(
						on_err(
							current_value.inner(),
							current_value.causes().preferred(),
							value,
						),
						cause,
					))
				}
			}
			None => {
				self.value = Some(WithCauses::new(value, cause));
				Ok(())
			}
		}
	}

	pub fn causes(&self) -> Option<&Causes<F>> {
		self.value.as_ref().map(|value| value.causes())
	}

	pub fn preferred_cause(&self) -> Option<&Location<F>> {
		self.value
			.as_ref()
			.and_then(|value| value.causes().preferred())
	}

	pub fn with_causes(&self) -> Option<&WithCauses<T, F>> {
		self.value.as_ref()
	}

	pub fn with_causes_mut(&mut self) -> Option<&mut WithCauses<T, F>> {
		self.value.as_mut()
	}

	pub fn value(&self) -> Option<&T> {
		self.value.as_ref().map(|v| v.inner())
	}

	pub fn value_mut(&mut self) -> Option<&mut T> {
		self.value.as_mut().map(|v| v.inner_mut())
	}

	pub fn into_value(self) -> Option<T> {
		self.value.map(WithCauses::into_inner)
	}

	pub fn as_deref(&self) -> Option<&T::Target>
	where
		T: std::ops::Deref,
	{
		self.value.as_ref().map(|v| v.inner().deref())
	}

	pub fn unwrap(self) -> Option<WithCauses<T, F>> {
		self.value
	}

	pub fn unwrap_or(self, default: T) -> WithCauses<T, F> {
		self.value
			.unwrap_or_else(|| WithCauses::without_causes(default))
	}

	pub fn unwrap_or_else_try<E>(
		self,
		f: impl FnOnce() -> Result<T, E>,
	) -> Result<WithCauses<T, F>, E> {
		self.value
			.map(Ok)
			.unwrap_or_else(|| f().map(WithCauses::without_causes))
	}

	pub fn or(self, other: Self) -> Self {
		if self.is_set() {
			self
		} else {
			other
		}
	}

	pub fn or_else(self, other: impl FnOnce() -> Self) -> Self {
		if self.is_set() {
			self
		} else {
			other()
		}
	}

	pub fn ok_or_else<E>(self, f: impl FnOnce() -> E) -> Result<WithCauses<T, F>, E> {
		self.value.ok_or_else(f)
	}

	pub fn value_or_else<E>(&self, f: impl FnOnce() -> E) -> Result<&WithCauses<T, F>, E> {
		self.value.as_ref().ok_or_else(f)
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> MaybeSet<U, F> {
		MaybeSet {
			value: self.value.map(|t| t.map(f)),
		}
	}

	pub fn try_map<U, E>(self, f: impl FnOnce(T) -> Result<U, E>) -> Result<MaybeSet<U, F>, E> {
		Ok(MaybeSet {
			value: self.value.map(|t| t.try_map(f)).transpose()?,
		})
	}

	pub fn map_with_causes<U>(self, f: impl FnOnce(WithCauses<T, F>) -> U) -> MaybeSet<U, F>
	where
		F: Clone,
	{
		MaybeSet {
			value: self.value.map(|t| {
				let causes = t.causes().clone();
				WithCauses::new(f(t), causes)
			}),
		}
	}

	pub fn try_map_with_causes<U, E>(
		self,
		f: impl FnOnce(T, &Causes<F>) -> Result<U, E>,
	) -> Result<MaybeSet<U, F>, E> {
		let value = match self.value {
			Some(t) => {
				let (t, causes) = t.into_parts();
				let u = f(t, &causes)?;
				Some(WithCauses::new(u, causes))
			}
			None => None,
		};

		Ok(MaybeSet { value })
	}
}

impl<T, F> From<WithCauses<T, F>> for MaybeSet<T, F> {
	fn from(t: WithCauses<T, F>) -> Self {
		Self::from(Some(t))
	}
}
