use crate::{Caused, Causes, Documentation, Id, MaybeSet, WithCauses, error, Error};
use locspan::Location;
use std::collections::HashMap;

/// Property definition.
pub struct Definition<F> {
	id: Id,
	domain: HashMap<Id, Causes<F>>,
	range: MaybeSet<Id, F>,
	required: MaybeSet<bool, F>,
	functional: MaybeSet<bool, F>,
	doc: Documentation,
}

impl<F> Definition<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			domain: HashMap::new(),
			range: MaybeSet::default(),
			required: MaybeSet::default(),
			functional: MaybeSet::default(),
			doc: Documentation::default(),
		}
	}

	pub fn range(&self) -> Option<&WithCauses<Id, F>> {
		self.range.with_causes()
	}

	pub fn is_required(&self) -> bool {
		self.required.value().cloned().unwrap_or(false)
	}

	pub fn set_required(&mut self, value: bool, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.required
			.try_set(value, cause, |expected, because, found| {
				error::PropertyMismatchRequired {
					id: self.id,
					expected: *expected,
					found,
					because: because.cloned(),
				}
				.into()
			})
	}

	/// Checks if this property is functional,
	/// meaning that it is associated to at most one value.
	pub fn is_functional(&self) -> bool {
		self.functional.value().cloned().unwrap_or(true)
	}

	pub fn set_functional(
		&mut self,
		value: bool,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.functional
			.try_set(value, cause, |expected, because, found| {
				error::PropertyMismatchFunctional {
					id: self.id,
					expected: *expected,
					found,
					because: because.cloned(),
				}
				.into()
			})
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn documentation_mut(&mut self) -> &mut Documentation {
		&mut self.doc
	}

	pub fn set_documentation(&mut self, doc: Documentation) {
		self.doc = doc
	}

	pub fn declare_domain(&mut self, ty_ref: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		use std::collections::hash_map::Entry;
		match self.domain.entry(ty_ref) {
			Entry::Vacant(entry) => {
				entry.insert(cause.into());
			}
			Entry::Occupied(mut entry) => {
				if let Some(cause) = cause {
					entry.get_mut().add(cause)
				}
			}
		}
	}

	pub fn declare_range(&mut self, ty: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.range.try_set(ty, cause, |expected, because, found| {
			error::PropertyMismatchType {
				id: self.id,
				expected: *expected,
				found,
				because: because.cloned(),
			}
			.into()
		})
	}
}

impl<F: Ord + Clone> WithCauses<Definition<F>, F> {
	pub fn build(
		self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
	) -> Result<crate::prop::Definition<F>, Error<F>> {
		let (def, causes) = self.into_parts();

		let range_id = def.range.ok_or_else(|| {
			Caused::new(
				error::PropertyMissingType(def.id).into(),
				causes.preferred().cloned(),
			)
		})?;
		let range = nodes
			.require_type(*range_id, range_id.causes().preferred().cloned())?
			.clone_with_causes(range_id.into_causes());

		let required = def.required.unwrap_or(false);
		let functional = def.functional.unwrap_or(true);

		let mut result = crate::prop::Definition::new(id, range, required, functional, causes);

		for (domain_id, domain_causes) in def.domain {
			let domain_ref = nodes.require_type(domain_id, domain_causes.preferred().cloned())?;
			result.insert_domain(*domain_ref.inner(), domain_causes)
		}

		Ok(result)
	}
}
