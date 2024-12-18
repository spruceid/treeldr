use std::hash::Hash;

use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_seeded::{unseeded, DeserializeSeeded, SerializeSeeded};

use crate::{
	pattern::serde_dataset_pattern,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	DatasetPattern, Pattern, TypeRef,
};

use super::Expr;

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe, SerializeSeeded, DeserializeSeeded,
)]
#[educe(Default)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct Bindings<P, R, T = TypeRef<P>> {
	#[seeded(with(unseeded))]
	pub variables: Vec<T>,

	/// Binding list.
	pub list: Vec<Binding<P, R, T>>,

	/// Dataset constraints.
	#[seeded(with(serde_dataset_pattern))]
	pub dataset: DatasetPattern<R>,
}

impl<P, R, T> Bindings<P, R, T> {
	pub fn is_empty(&self) -> bool {
		self.variables.is_empty()
	}

	pub fn len(&self) -> usize {
		self.variables.len()
	}
}

impl<P, R: Ord, T> Bindings<P, R, T> {
	pub fn map_types_ref_mut<U: Ord>(self, mut f: &mut impl FnMut(T) -> U) -> Bindings<P, R, U> {
		Bindings {
			variables: self.variables.into_iter().map(&mut f).collect(),
			list: self
				.list
				.into_iter()
				.map(|b| b.map_types_ref_mut(f))
				.collect(),
			dataset: self.dataset,
		}
	}
}

impl<P, R, T> VisitResources<R> for Bindings<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.list.visit_resources_ref_mut(f);
		self.dataset.visit_resources_ref_mut(f)
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct Binding<P, R, T = TypeRef<P>> {
	pub pattern: Pattern<P, R, T>,
	#[seeded(with(unseeded))]
	pub r#type: T,
	pub assignment: Option<Expr<P, R, T>>,
}

impl<P, R: Ord, T> Binding<P, R, T> {
	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Binding<P, R, U> {
		Binding {
			pattern: self.pattern.map_types_ref_mut(f),
			r#type: f(self.r#type),
			assignment: self.assignment.map(|a| a.map_types_ref_mut(f)),
		}
	}
}

impl<P, R, T> VisitResources<R> for Binding<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.assignment.visit_resources_ref_mut(f)
	}
}
