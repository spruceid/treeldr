use crate::{error, Context, Descriptions, Error};
use locspan::Meta;
use treeldr::{Documentation, Id, MetaOption, Name, Vocabulary};

/// Layout field definition.
#[derive(Clone)]
pub struct Definition<M> {
	id: Id,
	name: MetaOption<Name, M>,
	layout: MetaOption<Id, M>,
}

impl<M> Definition<M> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			name: MetaOption::default(),
			layout: MetaOption::default(),
		}
	}

	pub fn name(&self) -> Option<&Meta<Name, M>> {
		self.name.as_ref()
	}

	pub fn set_name(&mut self, name: Name, metadata: M) -> Result<(), Error<M>> {
		self.name
			.try_set(name, metadata, |Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::LayoutFieldMismatchName {
						id: self.id,
						expected,
						found,
						because: expected_meta,
					}
					.into(),
					found_meta,
				)
			})
	}

	pub fn replace_name(&mut self, name: MetaOption<Name, M>) {
		self.name = name
	}

	/// Build a default name for this layout variant.
	pub fn default_name<D: Descriptions<M>>(
		&self,
		context: &Context<M, D>,
		vocabulary: &Vocabulary,
		metadata: M,
	) -> Result<Option<Meta<Name, M>>, Error<M>>
	where
		M: Clone,
	{
		if let Id::Iri(term) = self.id {
			if let Ok(Some(name)) = Name::from_iri(term.iri(vocabulary).unwrap()) {
				return Ok(Some(Meta::new(name, metadata)));
			}
		}

		if let Some(layout_id) = self.layout.as_ref() {
			let layout = context
				.require_layout(**layout_id, layout_id.metadata())?;
			if let Some(name) = layout.name() {
				return Ok(Some(Meta::new(name.value().clone(), metadata)));
			}
		}

		Ok(None)
	}

	pub fn layout(&self) -> &MetaOption<Id, M> {
		&self.layout
	}

	pub fn set_layout(&mut self, layout_ref: Id, metadata: M) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.layout
			.try_set(layout_ref, metadata, |Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::LayoutFieldMismatchLayout {
						id: self.id,
						expected,
						found,
						because: expected_meta,
					}
					.into(),
					found_meta,
				)
			})
	}

	pub fn replace_layout(&mut self, layout: MetaOption<Id, M>) {
		self.layout = layout
	}
}

pub trait Build<M> {
	fn require_name(&self) -> Result<Meta<Name, M>, Error<M>>;

	fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		nodes: &super::super::context::allocated::Nodes<M>,
	) -> Result<treeldr::layout::Variant<M>, Error<M>>;
}

impl<M: Clone> Build<M> for Meta<Definition<M>, M> {
	fn require_name(&self) -> Result<Meta<Name, M>, Error<M>> {
		self.name.clone().ok_or_else(|| {
			Meta::new(
				error::LayoutVariantMissingName(self.id).into(),
				self.metadata().clone(),
			)
		})
	}

	fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		nodes: &super::super::context::allocated::Nodes<M>,
	) -> Result<treeldr::layout::Variant<M>, Error<M>> {
		let name = self.require_name()?;

		let layout = self
			.layout
			.clone()
			.try_map_with_causes(|Meta(layout_id, causes)| {
				Ok(Meta(**nodes.require_layout(layout_id, &causes)?, causes))
			})?;

		Ok(treeldr::layout::Variant::new(name, layout, label, doc))
	}
}
