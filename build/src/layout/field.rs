use crate::{Single, Context, resource::{self, BindingValueRef}, component::{AssertNamed, formatted::AssertFormatted}, prop, single, context::MapIds};

use super::Error;
use locspan::Meta;
use rdf_types::{Generator, Vocabulary, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, Id, IriIndex, Name};
pub use treeldr::layout::field::Property;

/// Layout field definition.
#[derive(Debug, Clone)]
pub struct Definition<M> {
	prop: Single<Id, M>
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id) -> Id) {
		self.prop.map_ids(f)
	}
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
				let layout = context.declare(id, meta.clone()).as_layout_mut();
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
		as_resource: &resource::Data<M>
	) -> Option<Meta<Name, M>>
	where
		M: Clone,
	{
		as_resource.id.as_iri()
			.and_then(|term| vocabulary.iri(term))
			.and_then(|iri| Name::from_iri(iri).ok().flatten())
			.map(|name| Meta::new(name, as_resource.metadata.clone()))
	}

	pub fn default_layout(
		&self,
		context: &crate::Context<M>,
	) -> Option<DefaultLayout<M>>
	where
		M: Clone,
	{
		let Meta(prop_id, _) = self.property().first()?;
		let prop = context.get(*prop_id)?;
		let range_id = prop.as_property().range().first()?;
		if prop.has_type(context, prop::Type::FunctionalProperty) {
			Some(DefaultLayout::Functional(range_id.cloned()))
		} else {
			Some(DefaultLayout::NonFunctional(range_id.cloned()))
		}
	}

	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings { prop: self.prop.iter() }
	}

	pub(crate) fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		as_component: &treeldr::component::Data<M>,
		as_formatted: &treeldr::component::formatted::Data<M>,
		meta: M
	) -> Result<Meta<treeldr::layout::field::Definition<M>, M>, Error<M>> where M: Clone {
		as_component.assert_named(as_resource, &meta)?;
		as_formatted.assert_formatted(as_resource, &meta)?;

		let prop =
			self.prop.clone()
				.into_property_at_node_binding(context, as_resource.id, Property::For)?;

		Ok(Meta(treeldr::layout::field::Definition::new(prop), meta))
	}
}

pub enum ClassBinding {
	For(Id)
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::For(_) => Property::For
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::For(v) => BindingValueRef::Id(*v)
		}
	}
}

pub struct ClassBindings<'a, M> {
	prop: single::Iter<'a, Id, M>
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.prop.next().map(|m| m.into_cloned_value().map(ClassBinding::For))
	}
}
