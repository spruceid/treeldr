use super::{error, Error};
use crate::{Caused, Causes, Documentation, Id, MaybeSet, Vocabulary, WithCauses, vocab::Name};
use locspan::Location;

/// Layout field definition.
pub struct Definition<F> {
	id: Id,
	prop: MaybeSet<Id, F>,
	name: MaybeSet<Name, F>,
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

	pub fn name(&self) -> Option<&WithCauses<Name, F>> {
		self.name.with_causes()
	}

	pub fn set_name(&mut self, name: Name, cause: Option<Location<F>>) -> Result<(), Error<F>>
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

	pub fn default_name(&self, vocab: &Vocabulary) -> Option<Name> {
		self.id.as_iri().and_then(|term| term.iri(vocab)).and_then(|iri| iri.path().file_name().and_then(|name| Name::try_from(name).ok()))
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

	pub fn require_layout(&self, causes: &Causes<F>) -> Result<&WithCauses<Id, F>, Error<F>> where F: Clone {
		self.layout.value_or_else(|| {
			Caused::new(
				error::LayoutFieldMissingLayout(self.id).into(),
				causes.preferred().cloned(),
			)
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
	pub fn require_name(&self, vocab: &Vocabulary) -> Result<WithCauses<Name, F>, Error<F>> where F: Clone {
		self.name.clone().unwrap_or_else_try(|| {
			self.default_name(vocab).ok_or_else(|| Caused::new(
				error::LayoutFieldMissingName(self.id).into(),
				self.causes().preferred().cloned(),
			))
		})
	}

	pub fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		vocab: &Vocabulary,
		nodes: &super::super::context::AllocatedNodes<F>,
	) -> Result<crate::layout::Field<F>, Error<F>> {
		let prop_id = self.prop.value_or_else(|| {
			Caused::new(
				error::LayoutFieldMissingProperty(self.id).into(),
				self.causes().preferred().cloned(),
			)
		})?;
		let prop = nodes
			.require_property(*prop_id.inner(), prop_id.causes().preferred().cloned())?
			.clone_with_causes(prop_id.causes().clone());

		let name = self.require_name(vocab)?;

		let layout_id = self.require_layout(self.causes())?;
		let layout = nodes
			.require_layout(*layout_id.inner(), layout_id.causes().preferred().cloned())?
			.clone_with_causes(layout_id.causes().clone());

		let required = self.required.clone().unwrap_or(false);
		let functional = self.functional.clone().unwrap_or(true);

		Ok(crate::layout::Field::new(
			prop, name, label, layout, required, functional, doc,
		))
	}
}
