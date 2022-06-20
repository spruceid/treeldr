use crate::{error, Context, Descriptions, Error};
use locspan::Location;
use treeldr::{Caused, Documentation, Id, MaybeSet, Name, Vocabulary, WithCauses};

/// Layout field definition.
#[derive(Clone)]
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
		self.name
			.try_set(name, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutFieldMismatchName {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}

	pub fn replace_name(&mut self, name: MaybeSet<Name, F>) {
		self.name = name
	}

	/// Build a default name for this layout variant.
	pub fn default_name<D: Descriptions<F>>(
		&self,
		context: &Context<F, D>,
		vocabulary: &Vocabulary,
		cause: Option<Location<F>>,
	) -> Result<Option<Caused<Name, F>>, Error<F>>
	where
		F: Clone,
	{
		if let Id::Iri(term) = self.id {
			if let Ok(Some(name)) = Name::from_iri(term.iri(vocabulary).unwrap()) {
				return Ok(Some(Caused::new(name, cause)));
			}
		}

		if let Some(layout_id) = self.layout.with_causes() {
			let layout = context
				.require_layout(*layout_id.inner(), layout_id.causes().preferred().cloned())?;
			if let Some(name) = layout.name() {
				return Ok(Some(Caused::new(name.inner().clone(), cause)));
			}
		}

		Ok(None)
	}

	pub fn layout(&self) -> &MaybeSet<Id, F> {
		&self.layout
	}

	pub fn set_layout(&mut self, layout_ref: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.layout
			.try_set(layout_ref, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutFieldMismatchLayout {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}

	pub fn replace_layout(&mut self, layout: MaybeSet<Id, F>) {
		self.layout = layout
	}
}

pub trait Build<F> {
	fn require_name(&self) -> Result<WithCauses<Name, F>, Error<F>>;

	fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		nodes: &super::super::context::allocated::Nodes<F>,
	) -> Result<treeldr::layout::Variant<F>, Error<F>>;
}

impl<F: Ord + Clone> Build<F> for WithCauses<Definition<F>, F> {
	fn require_name(&self) -> Result<WithCauses<Name, F>, Error<F>> {
		self.name.clone().ok_or_else(|| {
			Caused::new(
				error::LayoutVariantMissingName(self.id).into(),
				self.causes().preferred().cloned(),
			)
		})
	}

	fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		nodes: &super::super::context::allocated::Nodes<F>,
	) -> Result<treeldr::layout::Variant<F>, Error<F>> {
		let name = self.require_name()?;

		let layout = self
			.layout
			.clone()
			.try_map_with_causes(|layout_id, causes| {
				Ok(**nodes.require_layout(layout_id, causes.preferred().cloned())?)
			})?;

		Ok(treeldr::layout::Variant::new(name, layout, label, doc))
	}
}
