use std::cmp::Ordering;

use crate::{
	component::formatted::AssertFormatted,
	context::{MapIds, MapIdsIn},
	functional_property_value, prop, rdf,
	resource::{self, BindingValueRef},
	Context, FunctionalPropertyValue,
};

use super::Error;
use locspan::Meta;
use rdf_types::{Generator, Vocabulary, VocabularyMut};
pub use treeldr::layout::field::Property;
use treeldr::{
	metadata::Merge, prop::UnknownProperty, vocab::Object, BlankIdIndex, Id, IriIndex, Name,
	PropertyValueRef, TId,
};

/// Layout field definition.
#[derive(Debug, Clone)]
pub struct Definition<M> {
	prop: FunctionalPropertyValue<Id, M>,
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.prop.map_ids_in(Some(Property::For(None).into()), f)
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

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			prop: FunctionalPropertyValue::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn property(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.prop
	}

	pub fn property_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.prop
	}

	pub fn is_included_in(
		&self,
		context: &Context<M>,
		as_component: &crate::component::Data<M>,
		as_formatted: &crate::component::formatted::Data<M>,
		other: &Self,
		other_as_component: &crate::component::Data<M>,
		other_as_formatted: &crate::component::formatted::Data<M>,
	) -> bool {
		let common_prop = self.prop.iter().any(|a| {
			other
				.prop
				.iter()
				.any(|b| a.value.value() == b.value.value())
		});
		let no_prop = self.prop.is_empty() && other.prop.is_empty();

		let common_name = as_component.name.iter().any(|a| {
			other_as_component
				.name
				.iter()
				.any(|b| a.value.value() == b.value.value())
		});
		let no_name = as_component.name.is_empty() && other_as_component.name.is_empty();

		let included_layout = as_formatted.format.iter().all(|a| {
			other_as_formatted.format.iter().all(|b| {
				crate::layout::is_included_in(context, **a.value.value(), **b.value.value())
			})
		});

		(common_prop || no_prop) && (common_name || no_name) && included_layout
	}

	pub fn default_name(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		as_resource: &resource::Data<M>,
	) -> Option<Meta<Name, M>>
	where
		M: Clone,
	{
		as_resource
			.id
			.as_iri()
			.and_then(|term| vocabulary.iri(term))
			.and_then(|iri| Name::from_iri(iri).ok().flatten())
			.map(|name| Meta::new(name, as_resource.metadata.clone()))
	}

	pub fn default_layout(&self, context: &crate::Context<M>) -> Option<DefaultLayout<M>>
	where
		M: Clone,
	{
		let PropertyValueRef {
			value: Meta(prop_id, _),
			..
		} = self.property().first()?;
		let prop = context.get(*prop_id)?;
		let range_id = prop.as_property().range().first()?;
		if prop.has_type(context, prop::Type::FunctionalProperty) {
			Some(DefaultLayout::Functional(range_id.value.cloned()))
		} else {
			Some(DefaultLayout::NonFunctional(range_id.value.cloned()))
		}
	}

	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			prop: self.prop.iter(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		prop: Property,
		value: Meta<Object<M>, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::For(p) => self.prop.insert(p, prop_cmp, rdf::from::expect_id(value)?),
		}

		Ok(())
	}

	pub(crate) fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		_as_component: &treeldr::component::Data<M>,
		as_formatted: &treeldr::component::formatted::Data<M>,
		meta: M,
	) -> Result<Meta<treeldr::layout::field::Definition<M>, M>, Error<M>>
	where
		M: Clone,
	{
		as_formatted.assert_formatted(as_resource, &meta)?;

		let prop = self.prop.clone().into_property_at_node_binding(
			context,
			as_resource.id,
			Property::For(None),
		)?;

		Ok(Meta(treeldr::layout::field::Definition::new(prop), meta))
	}
}

pub fn is_included_in<M>(context: &Context<M>, a: Id, b: Id) -> bool {
	if a == b {
		true
	} else {
		let a = context.get(a).unwrap();
		let b = context.get(b).unwrap();
		a.as_layout_field().is_included_in(
			context,
			a.as_component().data(),
			a.as_formatted().data(),
			b.as_layout_field(),
			b.as_component().data(),
			b.as_formatted().data(),
		)
	}
}

#[derive(Debug)]
pub enum ClassBinding {
	For(Option<TId<UnknownProperty>>, Id),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::For(p, _) => Property::For(*p),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::For(_, v) => BindingValueRef::Id(*v),
		}
	}
}

pub struct ClassBindings<'a, M> {
	prop: functional_property_value::Iter<'a, Id, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.prop
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::For))
	}
}
