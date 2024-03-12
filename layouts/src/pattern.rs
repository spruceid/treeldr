use rdf_types::{pattern::ResourceOrVar, Quad};

/// A quad of patterns referencing their resources.
pub type PatternRefQuad<'p, R> = Quad<Pattern<&'p R>>;

/// Pattern.
///
/// Either a resource identifier or a variable.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Pattern<R> {
	/// Resource.
	Resource(R),

	/// Variable.
	Var(u32),
}

impl<R> Pattern<R> {
	pub fn apply(&self, substitution: &Substitution<R>) -> Self
	where
		R: Clone,
	{
		match self {
			Self::Resource(r) => Self::Resource(r.clone()),
			Self::Var(x) => match substitution.get(*x) {
				Some(r) => Self::Resource(r.clone()),
				None => Self::Var(*x),
			},
		}
	}

	pub fn as_ref(&self) -> Pattern<&R> {
		match self {
			Self::Resource(r) => Pattern::Resource(r),
			Self::Var(x) => Pattern::Var(*x),
		}
	}

	pub fn into_resource(self) -> Option<R> {
		match self {
			Self::Resource(r) => Some(r),
			_ => None,
		}
	}
}

impl<R> From<Pattern<R>> for ResourceOrVar<R, u32> {
	fn from(value: Pattern<R>) -> Self {
		match value {
			Pattern::Resource(r) => Self::Resource(r),
			Pattern::Var(x) => Self::Var(x),
		}
	}
}

/// Pattern substitution.
///
/// Maps some or all variables from `0` to [`Self::len()`] (called the
/// *declared* variables) to resources (`R`).
/// If all declared variables are bound to a resource, the substitution is
/// *complete*, or otherwise *partial*.
#[derive(Clone)]
pub struct Substitution<R>(Vec<Option<R>>);

impl<R> Substitution<R> {
	/// Create a new empty substitution without declared variables.
	pub fn new() -> Self {
		Self(Vec::new())
	}

	/// Creates a new substitution from the given input resources.
	///
	/// The resulting substitution has `inputs.len()` declared variables bound
	/// to there respective resource in `inputs`.
	pub fn from_inputs(inputs: &[R]) -> Self
	where
		R: Clone,
	{
		Self(inputs.iter().cloned().map(Some).collect())
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
	pub fn get(&self, i: u32) -> Option<&R> {
		self.0.get(i as usize).and_then(Option::as_ref)
	}

	/// Introduce `count` variables to the substitution. Returns the index of
	/// the first introduced variable.
	pub fn intro(&mut self, count: u32) -> u32 {
		let i = self.len();
		self.0.resize_with(self.0.len() + count as usize, || None);
		i
	}

	/// Introduce one more variable to the substitution and bind it to the given
	/// resource.
	///
	/// Returns the index of the newly declared variable.
	pub fn push(&mut self, value: Option<R>) -> u32 {
		let i = self.len();
		self.0.push(value);
		i
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
	pub fn set(&mut self, x: u32, value: Option<R>) -> Option<R> {
		std::mem::replace(&mut self.0[x as usize], value)
	}

	/// Copies and sets the variables of the substitution by matching the
	/// `value` quad against the given `pattern` quad.
	///
	/// For each `pattern` quad component:
	///   - if the pattern is a variable, sets the variable to the equivalent
	///     resource in `value` using [`Self::set`]. If the variable was already
	///     bound to another resource, a mismatch is found.
	///   - if the pattern is a resource, checks that it is equal to the
	///     corresponding resource in `value`, otherwise a mismatch is found.
	///
	/// Return the updated substitution if no mismatch is found, or `None`
	/// otherwise.
	pub fn with_quad(
		&self,
		pattern: Quad<Pattern<&R>, Pattern<&R>, Pattern<&R>, Pattern<&R>>,
		value: Quad<&R, &R, &R, &R>,
	) -> Option<Self>
	where
		R: Clone + PartialEq,
	{
		let mut result = self.clone();

		if let Pattern::Var(x) = pattern.0 {
			if let Some(old_value) = result.set(x, Some(value.0.clone())) {
				if old_value != *value.0 {
					return None;
				}
			}
		}

		if let Pattern::Var(x) = pattern.1 {
			if let Some(old_value) = result.set(x, Some(value.1.clone())) {
				if old_value != *value.1 {
					return None;
				}
			}
		}

		if let Pattern::Var(x) = pattern.2 {
			if let Some(old_value) = result.set(x, Some(value.2.clone())) {
				if old_value != *value.2 {
					return None;
				}
			}
		}

		if let Some(Pattern::Var(x)) = pattern.3 {
			let g = value.3.unwrap();
			if let Some(old_value) = result.set(x, Some(g.clone())) {
				if old_value != *g {
					return None;
				}
			}
		}

		Some(result)
	}
}

impl<R> Default for Substitution<R> {
	fn default() -> Self {
		Self::new()
	}
}
