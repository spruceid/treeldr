use std::collections::HashMap;

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
			if domain.len() > 1 {
				let mut domain = domain.clone();
				domain.keep_max_with(|&a, &b| self.subclass_partial_cmp(a.into(), b.into()));
				modified.insert(id, domain);
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
			if range.len() > 1 {
				let mut range = range.clone();
				range.keep_min_with(|&a, &b| self.subclass_partial_cmp(a.into(), b.into()));
				modified.insert(id, range);
			}
		}

		for (id, range) in modified {
			*self.get_mut(id).unwrap().as_property_mut().range_mut() = range;
		}
	}
}
