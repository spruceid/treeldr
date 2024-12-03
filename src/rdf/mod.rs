use std::collections::BTreeMap;

pub use iref;
pub use iref::{Iri, IriBuf};
pub use rdf_types::*;

mod context;
pub use context::*;

mod unify;
pub use unify::*;

mod substitution;
pub use substitution::*;

mod index;
pub use index::{ResourceBackwardIndex, ResourceForwardIndex};

pub mod serde;

pub trait VisitResources<R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R));

	fn visit_resources(&self, mut f: impl FnMut(&R)) {
		Self::visit_resources_ref_mut(self, &mut f);
	}
}

impl<'a, R, T: VisitResources<R>> VisitResources<R> for &'a T {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		T::visit_resources_ref_mut(*self, f)
	}
}

impl<R, T: VisitResources<R>> VisitResources<R> for Box<T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		T::visit_resources_ref_mut(&**self, f)
	}
}

impl<R, T: VisitResources<R>> VisitResources<R> for Option<T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		if let Some(t) = self {
			t.visit_resources_ref_mut(f)
		}
	}
}

impl<R, T: VisitResources<R>> VisitResources<R> for Vec<T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		for item in self {
			item.visit_resources_ref_mut(f)
		}
	}
}

impl<R, K, V> VisitResources<R> for BTreeMap<K, V>
where
	K: VisitResources<R>,
	V: VisitResources<R>,
{
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		for (key, value) in self {
			key.visit_resources_ref_mut(f);
			value.visit_resources_ref_mut(f);
		}
	}
}

impl<R, T: VisitResources<R>> VisitResources<R> for rdf_types::dataset::BTreeDataset<T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		for quad in self {
			quad.visit_resources_ref_mut(f)
		}
	}
}

impl<R, T: VisitResources<R>> VisitResources<R> for rdf_types::Quad<T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.0.visit_resources_ref_mut(f);
		self.1.visit_resources_ref_mut(f);
		self.2.visit_resources_ref_mut(f);
		self.3.visit_resources_ref_mut(f)
	}
}
