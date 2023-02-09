use std::cmp::Ordering;

use crate::{
	context::{MapIds, MapIdsIn},
	rdf,
	resource::BindingValueRef,
	Context, Error,
	functional_property_value,
	FunctionalPropertyValue
};
use locspan::Meta;
use treeldr::{metadata::Merge, value::NonNegativeInteger, vocab::Object, Id};

pub use treeldr::ty::restriction::{Cardinality, Property};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Range {
	Any(Id),
	All(Id),
}

impl MapIds for Range {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		match self {
			Self::Any(id) => id.map_ids_in(Some(Property::SomeValuesFrom.into()), f),
			Self::All(id) => id.map_ids_in(Some(Property::AllValuesFrom.into()), f),
		}
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Restriction {
	Range(Range),
	Cardinality(Cardinality),
}

impl MapIds for Restriction {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		if let Self::Range(r) = self {
			r.map_ids(f)
		}
	}
}

#[derive(Clone)]
pub struct Definition<M> {
	property: FunctionalPropertyValue<Id, M>,
	restriction: FunctionalPropertyValue<Restriction, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			property: FunctionalPropertyValue::default(),
			restriction: FunctionalPropertyValue::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn property(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.property
	}

	pub fn property_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.property
	}

	pub fn restriction(&self) -> &FunctionalPropertyValue<Restriction, M> {
		&self.restriction
	}

	pub fn restriction_mut(&mut self) -> &mut FunctionalPropertyValue<Restriction, M> {
		&mut self.restriction
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			on_property: self.property.iter(),
			restriction: self.restriction.iter(),
		}
	}

	pub fn set(&mut self, prop_cmp: impl Fn(Id, Id) -> Option<Ordering>, prop: Property, value: Meta<Object<M>, M>) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::OnProperty => self.property.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			Property::AllValuesFrom => self
				.restriction
				.insert(None, prop_cmp, rdf::from::expect_id(value)?.map(|id| Restriction::Range(Range::All(id)))),
			Property::SomeValuesFrom => self
				.restriction
				.insert(None, prop_cmp, rdf::from::expect_id(value)?.map(|id| Restriction::Range(Range::Any(id)))),
			Property::MaxCardinality => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_non_negative_integer(value)?
					.map(|n| Restriction::Cardinality(Cardinality::AtMost(n))),
			),
			Property::MinCardinality => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_non_negative_integer(value)?
					.map(|n| Restriction::Cardinality(Cardinality::AtLeast(n))),
			),
			Property::Cardinality => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_non_negative_integer(value)?
					.map(|n| Restriction::Cardinality(Cardinality::Exactly(n))),
			),
		}

		Ok(())
	}

	pub fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		meta: &M,
	) -> Result<treeldr::ty::restriction::Definition<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let prop_ref = self
			.property
			.clone()
			.into_required_property_at_node_binding(
				context,
				as_resource.id,
				Property::OnProperty,
				meta,
			)?;
		let restriction = self
			.restriction
			.clone()
			.try_unwrap()
			.map_err(|_| todo!())?
			.ok_or_else(|| todo!())?;

		Ok(treeldr::ty::restriction::Definition::new(
			prop_ref,
			restriction
				.into_value()
				.build(context, as_resource.id, meta.clone())?,
		))
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.property
			.map_ids_in(Some(Property::OnProperty.into()), &f);
		self.restriction.map_ids(f)
	}
}

impl Restriction {
	pub fn build<M>(
		self,
		context: &Context<M>,
		id: Id,
		meta: M,
	) -> Result<Meta<treeldr::ty::Restriction, M>, Error<M>>
	where
		M: Clone,
	{
		let r = match self {
			Self::Range(r) => treeldr::ty::Restriction::Range(r.build(context, id, &meta)?),
			Self::Cardinality(c) => treeldr::ty::Restriction::Cardinality(c),
		};

		Ok(Meta(r, meta))
	}

	pub fn as_binding_ref(&self) -> ClassBindingRef {
		match self {
			Self::Range(r) => r.as_binding_ref(),
			Self::Cardinality(r) => match r {
				Cardinality::AtLeast(v) => ClassBindingRef::MinCardinality(v),
				Cardinality::AtMost(v) => ClassBindingRef::MaxCardinality(v),
				Cardinality::Exactly(v) => ClassBindingRef::Cardinality(v),
			},
		}
	}
}

impl Range {
	pub fn build<M>(
		self,
		context: &Context<M>,
		id: Id,
		meta: &M,
	) -> Result<treeldr::ty::restriction::Range, Error<M>>
	where
		M: Clone,
	{
		match self {
			Self::Any(ty_id) => {
				let ty_ref = context
					.require_type_id(ty_id)
					.map_err(|e| e.at_node_property(id, Property::SomeValuesFrom, meta.clone()))?;
				Ok(treeldr::ty::restriction::Range::Any(ty_ref))
			}
			Self::All(ty_id) => {
				let ty_ref = context
					.require_type_id(ty_id)
					.map_err(|e| e.at_node_property(id, Property::AllValuesFrom, meta.clone()))?;
				Ok(treeldr::ty::restriction::Range::All(ty_ref))
			}
		}
	}

	pub fn as_binding_ref<'a>(&self) -> ClassBindingRef<'a> {
		match self {
			Self::Any(v) => ClassBindingRef::SomeValuesFrom(*v),
			Self::All(v) => ClassBindingRef::AllValuesFrom(*v),
		}
	}
}

pub enum ClassBinding {
	OnProperty(Id),
	SomeValuesFrom(Id),
	AllValuesFrom(Id),
	MinCardinality(NonNegativeInteger),
	MaxCardinality(NonNegativeInteger),
	Cardinality(NonNegativeInteger),
}

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::OnProperty(_) => Property::OnProperty,
			Self::SomeValuesFrom(_) => Property::SomeValuesFrom,
			Self::AllValuesFrom(_) => Property::AllValuesFrom,
			Self::MinCardinality(_) => Property::MinCardinality,
			Self::MaxCardinality(_) => Property::MaxCardinality,
			Self::Cardinality(_) => Property::Cardinality,
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<M> {
		match self {
			Self::OnProperty(v) => BindingValueRef::Id(*v),
			Self::SomeValuesFrom(v) => BindingValueRef::Id(*v),
			Self::AllValuesFrom(v) => BindingValueRef::Id(*v),
			Self::MinCardinality(v) => BindingValueRef::NonNegativeInteger(v),
			Self::MaxCardinality(v) => BindingValueRef::NonNegativeInteger(v),
			Self::Cardinality(v) => BindingValueRef::NonNegativeInteger(v),
		}
	}
}

pub enum ClassBindingRef<'a> {
	OnProperty(Id),
	SomeValuesFrom(Id),
	AllValuesFrom(Id),
	MinCardinality(&'a NonNegativeInteger),
	MaxCardinality(&'a NonNegativeInteger),
	Cardinality(&'a NonNegativeInteger),
}

impl<'a> ClassBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::OnProperty(_) => Property::OnProperty,
			Self::SomeValuesFrom(_) => Property::SomeValuesFrom,
			Self::AllValuesFrom(_) => Property::AllValuesFrom,
			Self::MinCardinality(_) => Property::MinCardinality,
			Self::MaxCardinality(_) => Property::MaxCardinality,
			Self::Cardinality(_) => Property::Cardinality,
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::OnProperty(v) => BindingValueRef::Id(*v),
			Self::SomeValuesFrom(v) => BindingValueRef::Id(*v),
			Self::AllValuesFrom(v) => BindingValueRef::Id(*v),
			Self::MinCardinality(v) => BindingValueRef::NonNegativeInteger(v),
			Self::MaxCardinality(v) => BindingValueRef::NonNegativeInteger(v),
			Self::Cardinality(v) => BindingValueRef::NonNegativeInteger(v),
		}
	}
}

pub type Binding = ClassBinding;

pub type BindingRef<'a> = ClassBindingRef<'a>;

pub struct ClassBindings<'a, M> {
	on_property: functional_property_value::Iter<'a, Id, M>,
	restriction: functional_property_value::Iter<'a, Restriction, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_property
			.next()
			.map(Meta::into_cloned_value)
			.map(|m| m.map(ClassBindingRef::OnProperty))
			.or_else(|| {
				self.restriction
					.next()
					.map(|m| m.map(Restriction::as_binding_ref))
			})
	}
}
