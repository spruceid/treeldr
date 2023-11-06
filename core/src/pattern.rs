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
