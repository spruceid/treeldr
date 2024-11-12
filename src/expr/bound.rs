use educe::Educe;

use crate::{DatasetPattern, TypeRef};

use super::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe)]
#[educe(Default)]
pub struct Bindings<R, T = TypeRef> {
	/// Introduced variables.
	pub intro: Vec<Binding<R, T>>,

	/// Dataset constraints.
	pub dataset: DatasetPattern<R>,
}

impl<R: Ord, T> Bindings<R, T> {
	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Bindings<R, U> {
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
pub struct Binding<R, T = TypeRef> {
	pub type_: T,
	pub assignment: Option<Expr<R, T>>,
}

impl<R: Ord, T> Binding<R, T> {
	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Binding<R, U> {
		Binding {
			type_: f(self.type_),
			assignment: self.assignment.map(|a| a.map_types_ref_mut(f)),
		}
	}
}
