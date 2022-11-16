use crate::{node, Single, component, Context, resource};

use super::Error;
use locspan::Meta;
use rdf_types::{Generator, Vocabulary, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, Id, IriIndex, Name};

/// Layout field definition.
#[derive(Clone)]
pub struct Definition<M> {
	prop: Single<Id, M>
}

pub enum DefaultLayout<M> {
	Functional(Meta<Id, M>),
	NonFunctional(Meta<Id, M>),
}

impl<M> DefaultLayout<M> {
	pub fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		context: &mut crate::Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Meta<Id, M>
	where
		M: Clone + Merge,
	{
		match self {
			Self::Functional(layout) => layout,
			Self::NonFunctional(Meta(item, meta)) => {
				let id = generator.next(vocabulary);
				context.declare_layout(id, meta.clone());
				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_array(Meta(item, meta.clone()));
				Meta(id, meta)
			}
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self {
			prop: Single::default()
		}
	}

	pub fn property(&self) -> &Single<Id, M> {
		&self.prop
	}

	pub fn property_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.prop
	}

	pub fn default_name(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		as_resource: &resource::Data<M>,
		metadata: M,
	) -> Option<Meta<Name, M>>
	where
		M: Clone,
	{
		as_resource.id.as_iri()
			.and_then(|term| vocabulary.iri(term))
			.and_then(|iri| Name::from_iri(iri).ok().flatten())
			.map(|name| Meta::new(name, metadata))
	}

	pub fn default_layout(
		&self,
		context: &crate::Context<M>,
	) -> Option<DefaultLayout<M>>
	where
		M: Clone,
	{
		self.property().first().and_then(|Meta(prop_id, _)| {
			let prop = context.get(*prop_id)?.as_property()?;
			let range_id = prop.range().first()?;

			let range_node = context.get(**range_id)?;
			if range_node.as_layout().is_some() {
				if prop.is_functional() {
					Some(DefaultLayout::Functional(range_id.cloned()))
				} else {
					Some(DefaultLayout::NonFunctional(range_id.cloned()))
				}
			} else {
				None
			}
		})
	}

	fn build(
		&self,
		context: &Context<M>,
		as_resource: &resource::Data<M>,
		as_component: &component::Data<M>,
		as_formatted: &component::formatted::Data<M>
	) -> Result<treeldr::layout::Field<M>, Error<M>> {
		let prop =
			self.prop.clone()
				.into_property_at_node_binding(context, as_resource.id, node::property::LayoutField::For)?;
		let layout = self.layout.clone().into_required_layout_at_node_binding(
			context,
			as_resource.id,
			node::property::Resource::Format,
			self.metadata(),
		)?;
		let name = self.name.clone().into_required_at_node_binding(
			as_resource.id,
			node::property::Resource::Name,
			self.metadata(),
		)?;

		Ok(treeldr::layout::Field::new(prop, layout))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	For,
	Name,
	Format,
}

pub enum BindingRef<'a> {
	For(Id),
	Name(&'a Name),
	Format(Id),
}

pub struct Bindings<'a> {
	prop: Option<Id>,
	name: Option<&'a Name>,
	format: Option<Id>,
}

impl<'a> Iterator for Bindings<'a> {
	type Item = BindingRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.prop.take().map(BindingRef::For).or_else(|| {
			self.name
				.take()
				.map(BindingRef::Name)
				.or_else(|| self.format.take().map(BindingRef::Format))
		})
	}
}
