use derivative::Derivative;
use locspan::Location;
use std::borrow::{Borrow, BorrowMut};
use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Caused<T, F> {
	t: T,
	cause: Option<Location<F>>,
}

impl<T, F> Caused<T, F> {
	pub fn new(t: T, cause: Option<Location<F>>) -> Self {
		Self { t, cause }
	}

	pub fn inner(&self) -> &T {
		&self.t
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.t
	}

	pub fn cause(&self) -> Option<&Location<F>> {
		self.cause.as_ref()
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Caused<U, F> {
		Caused::new(f(self.t), self.cause)
	}

	pub fn into_parts(self) -> (T, Option<Location<F>>) {
		(self.t, self.cause)
	}
}

impl<T, F> Caused<Caused<T, F>, F> {
	pub fn flatten(self) -> Caused<T, F> {
		if self.t.cause.is_none() {
			Caused::new(self.t.t, self.cause)
		} else {
			self.t
		}
	}
}

impl<T, F> Deref for Caused<T, F> {
	type Target = T;

	fn deref(&self) -> &T {
		self.inner()
	}
}

impl<T, F> DerefMut for Caused<T, F> {
	fn deref_mut(&mut self) -> &mut T {
		self.inner_mut()
	}
}

#[derive(Derivative)]
#[derivative(Clone, Default(bound = ""), Debug)]
pub struct Causes<F> {
	set: BTreeSet<Location<F>>,
}

impl<F> Causes<F> {
	pub fn new() -> Self {
		Self::default()
	}

	/// Adds a new cause.
	pub fn add(&mut self, cause: Location<F>)
	where
		F: Ord,
	{
		self.set.insert(cause);
	}

	pub fn with<I: IntoIterator<Item = Location<F>>>(mut self, other: I) -> Self
	where
		F: Ord,
	{
		self.extend(other);
		self
	}

	/// Picks the preferred cause, unless there are no causes.
	pub fn preferred(&self) -> Option<&Location<F>> {
		self.set.iter().next()
	}

	pub fn into_preferred(self) -> Option<Location<F>> {
		self.set.into_iter().next()
	}

	pub fn iter(&self) -> impl Iterator<Item = &Location<F>> {
		self.set.iter()
	}

	pub fn map(&self, f: impl Fn(Location<F>) -> Location<F>) -> Self
	where
		F: Clone + Ord,
	{
		Self {
			set: self.set.iter().cloned().map(f).collect(),
		}
	}
}

impl<F: Ord> From<Location<F>> for Causes<F> {
	fn from(cause: Location<F>) -> Self {
		let mut causes = Self::new();
		causes.add(cause);
		causes
	}
}

impl<F: Ord> From<Option<Location<F>>> for Causes<F> {
	fn from(cause: Option<Location<F>>) -> Self {
		let mut causes = Self::new();
		if let Some(cause) = cause {
			causes.add(cause);
		}
		causes
	}
}

impl<F: Ord> Extend<Location<F>> for Causes<F> {
	fn extend<I: IntoIterator<Item = Location<F>>>(&mut self, iter: I) {
		for cause in iter {
			self.add(cause);
		}
	}
}

impl<F: Ord> Extend<Option<Location<F>>> for Causes<F> {
	fn extend<I: IntoIterator<Item = Option<Location<F>>>>(&mut self, iter: I) {
		for cause in iter.into_iter().flatten() {
			self.add(cause)
		}
	}
}

impl<F> IntoIterator for Causes<F> {
	type Item = Location<F>;
	type IntoIter = std::collections::btree_set::IntoIter<Location<F>>;

	fn into_iter(self) -> Self::IntoIter {
		self.set.into_iter()
	}
}

#[derive(Clone, Debug)]
pub struct WithCauses<T, F> {
	t: T,
	causes: Causes<F>,
}

impl<T, F> WithCauses<T, F> {
	pub fn new(t: T, causes: impl Into<Causes<F>>) -> Self {
		Self {
			t,
			causes: causes.into(),
		}
	}

	pub fn without_causes(t: T) -> Self {
		Self {
			t,
			causes: Causes::new(),
		}
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn add_cause(&mut self, cause: Location<F>)
	where
		F: Ord,
	{
		self.causes.add(cause)
	}

	pub fn add_opt_cause(&mut self, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		if let Some(cause) = cause {
			self.causes.add(cause)
		}
	}

	pub fn inner(&self) -> &T {
		&self.t
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.t
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> WithCauses<U, F> {
		WithCauses::new(f(self.t), self.causes)
	}

	pub fn try_map<U, E>(
		self,
		f: impl FnOnce(T) -> Result<U, E>,
	) -> Result<WithCauses<U, F>, Caused<E, F>> {
		match f(self.t) {
			Ok(value) => Ok(WithCauses::new(value, self.causes)),
			Err(e) => Err(Caused::new(e, self.causes.into_preferred())),
		}
	}

	pub fn try_map_with_causes<U, E, M>(self, f: M) -> Result<WithCauses<U, F>, Caused<E, F>>
	where
		M: FnOnce(T, &Causes<F>) -> Result<U, E>,
	{
		match f(self.t, &self.causes) {
			Ok(value) => Ok(WithCauses::new(value, self.causes)),
			Err(e) => Err(Caused::new(e, self.causes.into_preferred())),
		}
	}

	pub fn clone_with_causes(&self, causes: impl Into<Causes<F>>) -> WithCauses<T, F>
	where
		T: Clone,
	{
		WithCauses {
			t: self.t.clone(),
			causes: causes.into(),
		}
	}

	pub fn into_inner(self) -> T {
		self.t
	}

	pub fn into_parts(self) -> (T, Causes<F>) {
		(self.t, self.causes)
	}

	pub fn into_causes(self) -> Causes<F> {
		self.causes
	}
}

impl<T, F> From<T> for WithCauses<T, F> {
	fn from(t: T) -> Self {
		Self::without_causes(t)
	}
}

impl<T, F> Deref for WithCauses<T, F> {
	type Target = T;

	fn deref(&self) -> &T {
		self.inner()
	}
}

impl<T, F> DerefMut for WithCauses<T, F> {
	fn deref_mut(&mut self) -> &mut T {
		self.inner_mut()
	}
}

impl<T, F> Borrow<T> for WithCauses<T, F> {
	fn borrow(&self) -> &T {
		self.inner()
	}
}

impl<T, F> BorrowMut<T> for WithCauses<T, F> {
	fn borrow_mut(&mut self) -> &mut T {
		self.inner_mut()
	}
}
