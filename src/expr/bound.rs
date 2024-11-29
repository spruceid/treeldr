use educe::Educe;

use crate::{DatasetPattern, TypeRef};

use super::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe)]
#[educe(Default)]
pub struct Bindings<P, R, T = TypeRef<P>> {
	/// Introduced variables.
	pub intro: Vec<Binding<P, R, T>>,

	/// Dataset constraints.
	pub dataset: DatasetPattern<R>,
}

impl<P, R: Ord, T> Bindings<P, R, T> {
	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Bindings<P, R, U> {
		Bindings {
			intro: self
				.intro
				.into_iter()
				.map(|b| b.map_types_ref_mut(f))
				.collect(),
			dataset: self.dataset,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Binding<P, R, T = TypeRef<P>> {
	pub type_: T,
	pub assignment: Option<Expr<P, R, T>>,
}

impl<P, R: Ord, T> Binding<P, R, T> {
	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Binding<P, R, U> {
		Binding {
			type_: f(self.type_),
			assignment: self.assignment.map(|a| a.map_types_ref_mut(f)),
		}
	}
}
