use super::{error, Error};
use crate::{Caused, Id, MaybeSet, Vocabulary, WithCauses};
use locspan::Location;

/// Layout field definition.
pub struct Definition<F> {
	id: Id,
	prop: MaybeSet<Id, F>,
	name: MaybeSet<String, F>,
	layout: MaybeSet<Id, F>,
	required: MaybeSet<bool, F>,
	functional: MaybeSet<bool, F>,
}

impl<F> Definition<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			prop: MaybeSet::default(),
			name: MaybeSet::default(),
			layout: MaybeSet::default(),
			required: MaybeSet::default(),
			functional: MaybeSet::default(),
		}
	}

	pub fn property(&self) -> Option<&WithCauses<Id, F>> {
		self.prop.with_causes()
	}

	pub fn set_property(&mut self, prop_ref: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.prop
			.try_set(prop_ref, cause, |expected, because, found| {
				error::LayoutFieldMismatchProperty {
					id: self.id,
					expected: *expected,
					found,
					because: because.cloned(),
				}
				.into()
			})
	}

	pub fn name(&self) -> Option<&WithCauses<String, F>> {
		self.name.with_causes()
	}

	pub fn set_name(&mut self, name: String, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.name.try_set(name, cause, |expected, because, found| {
			error::LayoutFieldMismatchName {
				id: self.id,
				expected: expected.clone(),
				found,
				because: because.cloned(),
			}
			.into()
		})
	}

	pub fn layout(&self) -> Option<&WithCauses<Id, F>> {
		self.layout.with_causes()
	}

	pub fn set_layout(&mut self, layout_ref: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.layout
			.try_set(layout_ref, cause, |expected, because, found| {
				error::LayoutFieldMismatchLayout {
					id: self.id,
					expected: *expected,
					found,
					because: because.cloned(),
				}
				.into()
			})
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
				error::LayoutFieldMismatchRequired {
					id: self.id,
					expected: *expected,
					found,
					because: because.cloned(),
				}
				.into()
			})
	}

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
				error::LayoutFieldMismatchFunctional {
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
		&self,
		id: Id,
		vocab: &Vocabulary,
		nodes: &super::super::context::AllocatedNodes<F>,
	) -> Result<crate::layout::Field<F>, Error<F>> {
		let prop_id = self.prop.value_or_else(|| {
			Caused::new(
				error::LayoutFieldMissingProperty(id).into(),
				self.causes().preferred().cloned(),
			)
		})?;
		let prop = nodes
			.require_property(*prop_id.inner(), prop_id.causes().preferred().cloned())?
			.clone_with_causes(prop_id.causes().clone());

		let name = self.name.clone().unwrap_or_else_try(|| match id {
			Id::Iri(name) => {
				let iri = name.iri(vocab).unwrap();
				Ok(iri
					.path()
					.file_name()
					.ok_or_else(|| {
						Caused::new(
							error::LayoutFieldMissingName(id).into(),
							self.causes().preferred().cloned(),
						)
					})?
					.into())
			}
			Id::Blank(_) => Err(Caused::new(
				error::LayoutFieldMissingName(id).into(),
				self.causes().preferred().cloned(),
			)),
		})?;

		let layout_id = self.layout.value_or_else(|| {
			Caused::new(
				error::LayoutFieldMissingLayout(id).into(),
				self.causes().preferred().cloned(),
			)
		})?;
		let layout = nodes
			.require_layout(*layout_id.inner(), layout_id.causes().preferred().cloned())?
			.clone_with_causes(layout_id.causes().clone());

		let required = self.required.clone().unwrap_or(false);
		let functional = self.functional.clone().unwrap_or(true);

		Ok(crate::layout::Field::new(
			prop, name, layout, required, functional,
		))
	}
}
