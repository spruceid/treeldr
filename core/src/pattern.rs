/// Pattern.
///
/// Either a resource identifier or a variable.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
		todo!()
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

#[derive(Clone)]
pub struct Substitution<R>(Vec<Option<R>>);

impl<R> Substitution<R> {
	pub fn new() -> Self {
		Self(Vec::new())
	}

	pub fn from_inputs(inputs: &[R]) -> Self
	where
		R: Clone,
	{
		Self(inputs.iter().cloned().map(Some).collect())
	}

	pub fn len(&self) -> u32 {
		self.0.len() as u32
	}

	pub fn get(&self, i: u32) -> Option<&R> {
		self.0.get(i as usize).map(Option::as_ref).flatten()
	}

	/// Introduce `count` variables to the substitution. Returns the index of
	/// the first introduced variable.
	pub fn intro(&mut self, count: u32) -> u32 {
		let i = self.len();
		self.0.resize_with(self.0.len() + count as usize, || None);
		i
	}

	pub fn push(&mut self, value: Option<R>) -> u32 {
		let i = self.len();
		self.0.push(value);
		i
	}

	pub fn set(&mut self, x: u32, value: Option<R>) {
		self.0[x as usize] = value
	}
}

impl<R> Default for Substitution<R> {
	fn default() -> Self {
		Self::new()
	}
}
