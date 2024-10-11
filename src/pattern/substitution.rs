use crate::Value;
use std::cell::OnceCell;

/// Pattern substitution.
///
/// Maps some or all variables from `0` to [`Self::len()`] to typed values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution<R>(Vec<OnceCell<Value<R>>>);

impl<R> Substitution<R> {
	/// Create a new empty substitution without declared variables.
	pub fn new(len: u32, f: impl Fn(u32) -> Option<Value<R>>) -> Self {
		let mut result = Vec::with_capacity(len as usize);

		for i in 0..len {
			let cell = OnceCell::new();

			if let Some(value) = f(i) {
				let _ = cell.set(value);
			}

			result.push(cell)
		}

		Self(result)
	}

	/// Returns the number of variables declared in the substitution.
	pub fn len(&self) -> u32 {
		self.0.len() as u32
	}

	/// Checks if the substitution is empty (no declared variables).
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns the resource bound to the variable `i`, if any.
	pub fn get(&self, i: u32) -> Option<&Value<R>> {
		self.0.get(i as usize).and_then(OnceCell::get)
	}

	/// Sets the binding of the variable `x` to `value`.
	///
	/// The variable `x` *must* be declared in the substitution.
	///
	/// Returns the previous binding of the variable.
	///
	/// ## Panics
	///
	/// Panics if the variable `x` if not declared in the substitution.
	pub fn set(&self, x: u32, value: Value<R>) -> Result<(), Value<R>>
	where
		R: PartialEq,
	{
		// std::mem::replace(&mut self.0[x as usize], value)
		match self.0.get(x as usize).unwrap().set(value) {
			Ok(()) => Ok(()),
			Err(value) => {
				if *self.get(x).unwrap() == value {
					Ok(())
				} else {
					Err(value)
				}
			}
		}
	}

	pub fn into_total(self) -> Result<Vec<Value<R>>, PartialSubstitution> {
		self.try_into_total_with(|_| Err(PartialSubstitution))
	}

	pub fn try_into_total_with<E>(
		self,
		mut f: impl FnMut(u32) -> Result<Value<R>, E>,
	) -> Result<Vec<Value<R>>, E> {
		let mut result = Vec::with_capacity(self.0.len() as usize);

		for (i, value) in self.0.into_iter().enumerate() {
			result.push(match value.into_inner() {
				None => f(i as u32)?,
				Some(value) => value,
			})
		}

		Ok(result)
	}
}

#[derive(Debug, thiserror::Error)]
#[error("partial substitution")]
pub struct PartialSubstitution;
