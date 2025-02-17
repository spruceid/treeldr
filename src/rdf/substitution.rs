use std::collections::BTreeMap;

use educe::Educe;
use rdf_types::{dataset::IndexedBTreeDataset, generator, BlankIdBuf, CowTerm, LocalTerm, Quad};

use crate::{eval::EvalContext, Value};

pub trait SubstituteResource<R = LocalTerm> {
	fn substitute_resource(&mut self, f: impl FnMut(R) -> R);
}

impl<R, T: SubstituteResource<R>> SubstituteResource<R> for Vec<T> {
	fn substitute_resource(&mut self, mut f: impl FnMut(R) -> R) {
		for t in self {
			t.substitute_resource(&mut f);
		}
	}
}

impl<R, T: SubstituteResource<R>> SubstituteResource<R> for &mut [T] {
	fn substitute_resource(&mut self, mut f: impl FnMut(R) -> R) {
		for t in self.iter_mut() {
			t.substitute_resource(&mut f);
		}
	}
}

impl<R: Clone + Ord> SubstituteResource<R> for IndexedBTreeDataset<R> {
	fn substitute_resource(&mut self, mut f: impl FnMut(R) -> R) {
		for Quad(s, p, o, g) in std::mem::take(self) {
			self.insert(Quad(f(s), f(p), f(o), g.map(&mut f)));
		}
	}
}

impl<R: Ord> SubstituteResource<R> for Value<R> {
	fn substitute_resource(&mut self, f: impl FnMut(R) -> R) {
		self.map_resources(f);
	}
}

#[derive(Educe)]
#[educe(Default)]
pub struct DebugSubstitution<R> {
	generator: generator::Blank,
	resources: BTreeMap<R, BlankIdBuf>,
	variables: BTreeMap<u32, BlankIdBuf>,
}

impl<R> DebugSubstitution<R> {
	pub fn new() -> Self {
		Self {
			generator: generator::Blank::new(),
			resources: BTreeMap::new(),
			variables: BTreeMap::new(),
		}
	}

	pub fn new_resource(&mut self) -> LocalTerm {
		LocalTerm::Anonymous(self.generator.next_blank_id())
	}
}

impl<R: Ord> DebugSubstitution<R> {
	pub fn insert_resource(&mut self, rdf: &impl EvalContext<Resource = R>, r: &R) -> LocalTerm
	where
		R: Clone,
	{
		rdf.terms_of(r)
			.next()
			.map(CowTerm::into_owned)
			.map(LocalTerm::Named)
			.unwrap_or_else(|| {
				LocalTerm::Anonymous(
					self.resources
						.entry(r.clone())
						.or_insert_with(|| self.generator.next_blank_id())
						.clone(),
				)
			})
	}

	pub fn insert_var(&mut self, x: u32) -> LocalTerm {
		LocalTerm::Anonymous(
			self.variables
				.entry(x)
				.or_insert_with(|| self.generator.next_blank_id())
				.clone(),
		)
	}
}
