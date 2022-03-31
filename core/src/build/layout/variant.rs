use super::{error, Error};
use crate::{vocab::Name, Caused, Documentation, Id, MaybeSet, Vocabulary, WithCauses};
use locspan::Location;

/// Layout field definition.
pub struct Definition<F> {
	id: Id,
	name: MaybeSet<Name, F>,
	layout: MaybeSet<Id, F>,
}

impl<F> Definition<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			name: MaybeSet::default(),
			layout: MaybeSet::default(),
		}
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
		self.id
			.as_iri()
			.and_then(|term| term.iri(vocab))
			.and_then(|iri| {
				iri.path()
					.file_name()
					.and_then(|name| Name::try_from(name).ok())
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
}

impl<F: Ord + Clone> WithCauses<Definition<F>, F> {
	pub fn require_name(&self, vocab: &Vocabulary) -> Result<WithCauses<Name, F>, Error<F>>
	where
		F: Clone,
	{
		self.name.clone().unwrap_or_else_try(|| {
			self.default_name(vocab).ok_or_else(|| {
				Caused::new(
					error::LayoutFieldMissingName(self.id).into(),
					self.causes().preferred().cloned(),
				)
			})
		})
	}

	pub fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		vocab: &Vocabulary,
		nodes: &super::super::context::AllocatedNodes<F>,
	) -> Result<crate::layout::Variant<F>, Error<F>> {
		let name = self.require_name(vocab)?;

		let layout = self.layout.clone().try_map_with_causes(|layout_id| {
			Ok(*nodes
				.require_layout(*layout_id.inner(), layout_id.causes().preferred().cloned())?
				.inner())
		})?;

		Ok(crate::layout::Variant::new(name, layout, label, doc))
	}
}
