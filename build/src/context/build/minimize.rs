use std::collections::HashMap;

use locspan::Meta;
use treeldr::{PropertyValues, Type};

use crate::Context;

impl<M> Context<M> {
	pub(crate) fn minimize_domain_and_range(&mut self)
	where
		M: Clone,
	{
		self.minimize_domain();
		self.minimize_range();
	}

	pub(crate) fn minimize_domain(&mut self)
	where
		M: Clone,
	{
		let mut modified = HashMap::new();
		for (id, n) in self.nodes() {
			let domain = n.as_property().domain();
			match domain.len() {
				0 => {
					let mut domain = PropertyValues::default();
					domain.insert_base_unique(Meta(
						Type::Resource(None).into_raw_id(),
						n.metadata().clone(),
					));
					modified.insert(id, domain);
				}
				1 => (),
				_ => {
					let mut domain = domain.clone();
					domain.keep_max_with(|&a, &b| self.subclass_partial_cmp(a.into(), b.into()));
					modified.insert(id, domain);
				}
			}
		}

		for (id, domain) in modified {
			*self.get_mut(id).unwrap().as_property_mut().domain_mut() = domain;
		}
	}

	pub(crate) fn minimize_range(&mut self)
	where
		M: Clone,
	{
		let mut modified = HashMap::new();
		for (id, n) in self.nodes() {
			let range = n.as_property().range();
			match range.len() {
				0 => {
					let mut range = PropertyValues::default();
					range.insert_base_unique(Meta(
						Type::Resource(None).into_raw_id(),
						n.metadata().clone(),
					));
					modified.insert(id, range);
				}
				1 => (),
				_ => {
					let mut range = range.clone();
					range.keep_min_with(|&a, &b| self.subclass_partial_cmp(a.into(), b.into()));
					modified.insert(id, range);
				}
			}
		}

		for (id, range) in modified {
			*self.get_mut(id).unwrap().as_property_mut().range_mut() = range;
		}
	}
}
