use std::collections::BTreeMap;

use educe::Educe;
use rdf_types::{generator, BlankIdBuf, Term};

use super::RdfContext;

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

	pub fn new_resource(&mut self) -> Term {
		Term::blank(self.generator.next_blank_id())
	}
}

impl<R: Ord> DebugSubstitution<R> {
	pub fn insert_resource(&mut self, rdf: &impl RdfContext<Resource = R>, r: &R) -> Term
	where
		R: Clone,
	{
		rdf.term_of(r).unwrap_or_else(|| {
			Term::blank(
				self.resources
					.entry(r.clone())
					.or_insert_with(|| self.generator.next_blank_id())
					.clone(),
			)
		})
	}

	pub fn insert_var(&mut self, x: u32) -> Term {
		Term::blank(
			self.variables
				.entry(x)
				.or_insert_with(|| self.generator.next_blank_id())
				.clone(),
		)
	}
}
