use super::{error, Error};
use locspan::Meta;
use rdf_types::Vocabulary;
use treeldr::{
	metadata::Merge, to_rdf::Generator, BlankIdIndex, Documentation, Id, IriIndex, MetaOption, Name,
};

/// Layout field definition.
#[derive(Clone)]
pub struct Definition<M> {
	id: Id,
	prop: MetaOption<Id, M>,
	name: MetaOption<Name, M>,
	layout: MetaOption<Id, M>,
}

pub enum DefaultLayout<M> {
	Functional(Meta<Id, M>),
	NonFunctional(Meta<Id, M>),
}

impl<M> DefaultLayout<M> {
	pub fn build<D: crate::Descriptions<M>>(
		self,
		context: &mut crate::Context<M, D>,
		vocabulary: &mut impl Generator,
	) -> Meta<Id, M>
	where
		M: Clone + Merge,
	{
		match self {
			Self::Functional(layout) => layout,
			Self::NonFunctional(Meta(item, meta)) => {
				let id = vocabulary.next();
				context.declare_layout(id, meta.clone());
				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();

				layout.set_array(item, None, meta.clone()).ok();

				Meta(id, meta)
			}
		}
	}
}

impl<M> Definition<M> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			prop: MetaOption::default(),
			name: MetaOption::default(),
			layout: MetaOption::default(),
		}
	}

	pub fn property(&self) -> Option<&Meta<Id, M>> {
		self.prop.as_ref()
	}

	pub fn set_property(&mut self, prop_ref: Id, metadata: M) -> Result<(), Error<M>> {
		self.prop.try_set(
			prop_ref,
			metadata,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::LayoutFieldMismatchProperty {
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

	pub fn replace_property(&mut self, prop: MetaOption<Id, M>) {
		self.prop = prop
	}

	pub fn default_name(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		metadata: M,
	) -> Option<Meta<Name, M>>
	where
		M: Clone,
	{
		self.id
			.as_iri()
			.and_then(|term| vocabulary.iri(term))
			.and_then(|iri| Name::from_iri(iri).ok().flatten())
			.map(|name| Meta::new(name, metadata))
	}

	pub fn name(&self) -> Option<&Meta<Name, M>> {
		self.name.as_ref()
	}

	pub fn set_name(&mut self, name: Name, metadata: M) -> Result<(), Error<M>> {
		self.name.try_set(
			name,
			metadata,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
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
			},
		)
	}

	pub fn replace_name(&mut self, name: MetaOption<Name, M>) {
		self.name = name
	}

	pub fn layout(&self) -> Option<&Meta<Id, M>> {
		self.layout.as_ref()
	}

	pub fn set_layout(&mut self, layout_ref: Id, metadata: M) -> Result<(), Error<M>> {
		self.layout.try_set(
			layout_ref,
			metadata,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
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
			},
		)
	}

	pub fn replace_layout(&mut self, layout: MetaOption<Id, M>) {
		self.layout = layout
	}

	pub fn default_layout<D: crate::Descriptions<M>>(
		&self,
		context: &crate::Context<M, D>,
	) -> Option<DefaultLayout<M>>
	where
		M: Clone,
	{
		let prop_id = self.property()?;
		let prop = context.get(**prop_id)?.as_property()?;
		let range_id = prop.range()?;

		let range_node = context.get(**range_id)?;
		if range_node.as_layout().is_some() {
			if prop.is_functional() {
				Some(DefaultLayout::Functional(range_id.clone()))
			} else {
				Some(DefaultLayout::NonFunctional(range_id.clone()))
			}
		} else {
			None
		}
	}

	pub fn require_layout(&self, causes: &M) -> Result<&Meta<Id, M>, Error<M>>
	where
		M: Clone,
	{
		self.layout.value_or_else(|| {
			Error::new(
				error::LayoutFieldMissingLayout(self.id).into(),
				causes.clone(),
			)
		})
	}
}

pub trait Build<M> {
	fn require_name(&self) -> Result<Meta<Name, M>, Error<M>>;

	fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		nodes: &super::super::context::allocated::Nodes<M>,
	) -> Result<treeldr::layout::Field<M>, Error<M>>;
}

impl<M: Clone> Build<M> for Meta<Definition<M>, M> {
	fn require_name(&self) -> Result<Meta<Name, M>, Error<M>> {
		self.name.clone().ok_or_else(|| {
			Meta::new(
				error::LayoutFieldMissingName(self.id).into(),
				self.metadata().clone(),
			)
		})
	}

	fn build(
		&self,
		label: Option<String>,
		doc: Documentation,
		nodes: &super::super::context::allocated::Nodes<M>,
	) -> Result<treeldr::layout::Field<M>, Error<M>> {
		let prop = self
			.prop
			.clone()
			.try_map_with_causes(|Meta(prop_id, causes)| {
				Ok(Meta(**nodes.require_property(prop_id, &causes)?, causes))
			})?;

		let name = self.require_name()?;

		let layout_id = self.require_layout(self.metadata())?;
		let layout = nodes
			.require_layout(**layout_id, layout_id.metadata())?
			.clone();

		Ok(treeldr::layout::Field::new(prop, name, label, layout, doc))
	}
}
