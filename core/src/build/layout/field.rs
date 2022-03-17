use crate::{Id, Causes, Caused, MaybeSet, WithCauses};
use super::Error;
use locspan::Location;

/// Layout field definition.
pub struct Definition<F> {
	prop: MaybeSet<Id, F>,
	name: MaybeSet<String, F>,
	layout: MaybeSet<Id, F>,
	required: MaybeSet<bool, F>,
	functional: MaybeSet<bool, F>
}

impl<F> Definition<F> {
	pub fn new() -> Self {
		Self {
			prop: MaybeSet::default(),
			name: MaybeSet::default(),
			layout: MaybeSet::default(),
			required: MaybeSet::default(),
			functional: MaybeSet::default()
		}
	}

	pub fn property(&self) -> Option<&WithCauses<Id, F>> {
		self.prop.with_causes()
	}

	pub fn set_property(&mut self, prop_ref: Id, cause: Option<Location<F>>) -> Result<(), Caused<Error<F>, F>> where F: Ord {
		self.prop.try_set(prop_ref, cause, |expected, because, found| todo!())
	}

	pub fn name(&self) -> Option<&WithCauses<String, F>> {
		self.name.with_causes()
	}

	pub fn set_name(&mut self, name: String, cause: Option<Location<F>>) -> Result<(), Caused<Error<F>, F>> where F: Ord {
		self.name.try_set(name, cause, |expected, because, found| todo!())
	}

	pub fn layout(&self) -> Option<&WithCauses<Id, F>> {
		self.layout.with_causes()
	}

	pub fn set_layout(&mut self, layout_ref: Id, cause: Option<Location<F>>) -> Result<(), Caused<Error<F>, F>> where F: Ord {
		self.layout.try_set(layout_ref, cause, |expected, because, found| todo!())
	}

	pub fn is_required(&self) -> bool {
		self.required.value().cloned().unwrap_or(false)
	}

	pub fn set_required(&mut self, value: bool, cause: Option<Location<F>>) -> Result<(), Caused<Error<F>, F>> where F: Ord {
		self.required.try_set(value, cause, |expected, because, found| todo!())
	}

	pub fn is_functional(&self) -> bool {
		self.functional.value().cloned().unwrap_or(true)
	}

	pub fn set_functional(&mut self, value: bool, cause: Option<Location<F>>) -> Result<(), Caused<Error<F>, F>> where F: Ord {
		self.functional.try_set(value, cause, |expected, because, found| todo!())
	}
}

impl<F: Ord + Clone> WithCauses<Definition<F>, F> {
	pub fn build(&self, nodes: &super::super::context::AllocatedNodes<F>) -> Result<crate::layout::Field<F>, Caused<Error<F>, F>> {
		todo!()
	}
}