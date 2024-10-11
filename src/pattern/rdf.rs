use rdf_types::{pattern::ResourceOrVar, Quad};

use crate::Value;

use super::Substitution;

pub type Graph<R = rdf_types::Term> = rdf_types::dataset::BTreeGraph<TermPattern<R>>;

pub type DatasetPattern<R = rdf_types::Term> = rdf_types::dataset::BTreeDataset<TermPattern<R>>;

/// A quad of patterns referencing their resources.
pub type PatternRefQuad<'p, R> = Quad<TermPattern<&'p R>>;

/// Pattern.
///
/// Either a resource identifier or a variable.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum TermPattern<R> {
	/// Resource.
	Resource(R),

	/// Variable.
	Var(u32),
}

impl<R> TermPattern<R> {
	pub fn apply(&self, substitution: &Substitution<R>) -> Self
	where
		R: Clone,
	{
		match self {
			Self::Resource(r) => Self::Resource(r.clone()),
			Self::Var(x) => match substitution.get(*x).and_then(Value::as_resource) {
				Some(r) => Self::Resource(r.clone()),
				None => Self::Var(*x),
			},
		}
	}

	pub fn as_ref(&self) -> TermPattern<&R> {
		match self {
			Self::Resource(r) => TermPattern::Resource(r),
			Self::Var(x) => TermPattern::Var(*x),
		}
	}

	pub fn into_resource(self) -> Option<R> {
		match self {
			Self::Resource(r) => Some(r),
			_ => None,
		}
	}
}

impl<R> From<TermPattern<R>> for ResourceOrVar<R, u32> {
	fn from(value: TermPattern<R>) -> Self {
		match value {
			TermPattern::Resource(r) => Self::Resource(r),
			TermPattern::Var(x) => Self::Var(x),
		}
	}
}
