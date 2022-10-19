use crate::{error, Error};
use locspan::Meta;
use std::collections::HashMap;
use treeldr::{metadata::Merge, Id, MetaOption};

/// Property definition.
#[derive(Clone)]
pub struct Definition<M> {
	id: Id,
	domain: HashMap<Id, M>,
	range: MetaOption<Id, M>,
	required: MetaOption<bool, M>,
	functional: MetaOption<bool, M>,
}

impl<M> Definition<M> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			domain: HashMap::new(),
			range: MetaOption::default(),
			required: MetaOption::default(),
			functional: MetaOption::default(),
		}
	}

	pub fn range(&self) -> Option<&Meta<Id, M>> {
		self.range.as_ref()
	}

	pub fn is_required(&self) -> bool {
		self.required.value().cloned().unwrap_or(false)
	}

	pub fn set_required(&mut self, value: bool, cause: M) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.required.try_set(
			value,
			cause,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::PropertyMismatchRequired {
						id: self.id,
						expected,
						found,
						because: expected_meta,
					}
					.into(),
					found_meta,
				)
			},
		)
	}

	/// Checks if this property is functional,
	/// meaning that it is associated to at most one value.
	pub fn is_functional(&self) -> bool {
		self.functional.value().cloned().unwrap_or(true)
	}

	pub fn set_functional(&mut self, value: bool, cause: M) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.functional.try_set(
			value,
			cause,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::PropertyMismatchFunctional {
						id: self.id,
						expected,
						found,
						because: expected_meta,
					}
					.into(),
					found_meta,
				)
			},
		)
	}

	pub fn set_domain(&mut self, ty_ref: Id, cause: M)
	where
		M: Merge,
	{
		use std::collections::hash_map::Entry;
		match self.domain.entry(ty_ref) {
			Entry::Vacant(entry) => {
				entry.insert(cause);
			}
			Entry::Occupied(mut entry) => entry.get_mut().merge_with(cause),
		}
	}

	pub fn set_range(&mut self, ty: Id, cause: M) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.range.try_set(
			ty,
			cause,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::PropertyMismatchType {
						id: self.id,
						expected,
						found,
						because: expected_meta,
					}
					.into(),
					found_meta,
				)
			},
		)
	}

	pub fn dependencies(
		&self,
		_nodes: &super::context::allocated::Nodes<M>,
		_causes: &M,
	) -> Result<Vec<crate::Item<M>>, Error<M>>
	where
		M: Clone,
	{
		Ok(Vec::new())
	}
}

impl<M: Clone> crate::Build<M> for Definition<M> {
	type Target = treeldr::prop::Definition<M>;

	fn build(
		self,
		nodes: &mut super::context::allocated::Nodes<M>,
		_dependencies: crate::Dependencies<M>,
		causes: M,
	) -> Result<Self::Target, Error<M>> {
		let range_id = self
			.range
			.ok_or_else(|| Meta(error::PropertyMissingType(self.id).into(), causes.clone()))?;
		let range = Meta(
			*nodes.require_type(*range_id, range_id.metadata())?.value(),
			range_id.into_metadata(),
		);

		let required = self.required.unwrap_or_else(|| Meta(false, causes.clone()));
		let functional = self
			.functional
			.unwrap_or_else(|| Meta(true, causes.clone()));

		let mut result =
			treeldr::prop::Definition::new(self.id, range, required, functional, causes);

		for (domain_id, domain_causes) in self.domain {
			let domain_ref = nodes.require_type(domain_id, &domain_causes)?;
			result.insert_domain(**domain_ref, domain_causes)
		}

		Ok(result)
	}
}
