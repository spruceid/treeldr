use crate::{error, Error};
use locspan::Location;
use std::collections::HashMap;
use treeldr::{Caused, Causes, Id, MaybeSet, WithCauses};

/// Property definition.
#[derive(Clone)]
pub struct Definition<F> {
	id: Id,
	domain: HashMap<Id, Causes<F>>,
	range: MaybeSet<Id, F>,
	required: MaybeSet<bool, F>,
	functional: MaybeSet<bool, F>,
}

impl<F> Definition<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			domain: HashMap::new(),
			range: MaybeSet::default(),
			required: MaybeSet::default(),
			functional: MaybeSet::default(),
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

	pub fn set_domain(&mut self, ty_ref: Id, cause: Option<Location<F>>)
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

	pub fn set_range(&mut self, ty: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
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

	pub fn dependencies(
		&self,
		_nodes: &super::context::allocated::Nodes<F>,
		_causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>>
	where
		F: Clone + Ord,
	{
		Ok(Vec::new())
	}
}

impl<F: Ord + Clone> crate::Build<F> for Definition<F> {
	type Target = treeldr::prop::Definition<F>;

	fn build(
		self,
		nodes: &mut super::context::allocated::Nodes<F>,
		_dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Error<F>> {
		let range_id = self.range.ok_or_else(|| {
			Caused::new(
				error::PropertyMissingType(self.id).into(),
				causes.preferred().cloned(),
			)
		})?;
		let range = nodes
			.require_type(*range_id, range_id.causes().preferred().cloned())?
			.clone_with_causes(range_id.into_causes());

		let required = self.required.unwrap_or(false);
		let functional = self.functional.unwrap_or(true);

		let mut result =
			treeldr::prop::Definition::new(self.id, range, required, functional, causes);

		for (domain_id, domain_causes) in self.domain {
			let domain_ref = nodes.require_type(domain_id, domain_causes.preferred().cloned())?;
			result.insert_domain(*domain_ref.inner(), domain_causes)
		}

		Ok(result)
	}
}
