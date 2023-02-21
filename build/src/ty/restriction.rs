use std::cmp::Ordering;

use crate::{
	context::{MapIds, MapIdsIn},
	functional_property_value,
	resource::BindingValueRef,
	single, Context, Error, FunctionalPropertyValue, MetaValueExt, Single,
};
use locspan::Meta;
use treeldr::{metadata::Merge, prop::UnknownProperty, value::NonNegativeInteger, Id, TId, Value};

pub use treeldr::ty::restriction::{Cardinality, Property};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Range {
	Any(Id),
	All(Id),
}

impl MapIds for Range {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		match self {
			Self::Any(id) => id.map_ids_in(Some(Property::SomeValuesFrom(None).into()), f),
			Self::All(id) => id.map_ids_in(Some(Property::AllValuesFrom(None).into()), f),
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
	restriction: Single<Restriction, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			property: FunctionalPropertyValue::default(),
			restriction: Single::default(),
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

	pub fn restriction(&self) -> &Single<Restriction, M> {
		&self.restriction
	}

	pub fn restriction_mut(&mut self) -> &mut Single<Restriction, M> {
		&mut self.restriction
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			on_property: self.property.iter(),
			restriction: self.restriction.iter(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		prop: Property,
		value: Meta<Value, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::OnProperty(p) => self.property.insert(p, prop_cmp, value.into_expected_id()?),
			Property::AllValuesFrom(_) => self.restriction.insert(
				value
					.into_expected_id()?
					.map(|id| Restriction::Range(Range::All(id))),
			),
			Property::SomeValuesFrom(_) => self.restriction.insert(
				value
					.into_expected_id()?
					.map(|id| Restriction::Range(Range::Any(id))),
			),
			Property::MaxCardinality(_) => self.restriction.insert(
				value
					.into_expected_non_negative_integer()?
					.map(|n| Restriction::Cardinality(Cardinality::AtMost(n))),
			),
			Property::MinCardinality(_) => self.restriction.insert(
				value
					.into_expected_non_negative_integer()?
					.map(|n| Restriction::Cardinality(Cardinality::AtLeast(n))),
			),
			Property::Cardinality(_) => self.restriction.insert(
				value
					.into_expected_non_negative_integer()?
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
				Property::OnProperty(None),
				meta,
			)?;
		let Meta(restriction, restriction_meta) = self
			.restriction
			.clone()
			.try_unwrap()
			.map_err(|_| todo!())?
			.ok_or_else(|| todo!())?;

		Ok(treeldr::ty::restriction::Definition::new(
			prop_ref,
			restriction.build(context, as_resource.id, restriction_meta)?,
		))
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.property
			.map_ids_in(Some(Property::OnProperty(None).into()), &f);
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
				Cardinality::AtLeast(v) => ClassBindingRef::MinCardinality(None, v),
				Cardinality::AtMost(v) => ClassBindingRef::MaxCardinality(None, v),
				Cardinality::Exactly(v) => ClassBindingRef::Cardinality(None, v),
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
				let ty_ref = context.require_type_id(ty_id).map_err(|e| {
					e.at_node_property(id, Property::SomeValuesFrom(None), meta.clone())
				})?;
				Ok(treeldr::ty::restriction::Range::Any(ty_ref))
			}
			Self::All(ty_id) => {
				let ty_ref = context.require_type_id(ty_id).map_err(|e| {
					e.at_node_property(id, Property::AllValuesFrom(None), meta.clone())
				})?;
				Ok(treeldr::ty::restriction::Range::All(ty_ref))
			}
		}
	}

	pub fn as_binding_ref<'a>(&self) -> ClassBindingRef<'a> {
		match self {
			Self::Any(v) => ClassBindingRef::SomeValuesFrom(None, *v),
			Self::All(v) => ClassBindingRef::AllValuesFrom(None, *v),
		}
	}
}

pub enum ClassBinding {
	OnProperty(Option<TId<UnknownProperty>>, Id),
	SomeValuesFrom(Option<TId<UnknownProperty>>, Id),
	AllValuesFrom(Option<TId<UnknownProperty>>, Id),
	MinCardinality(Option<TId<UnknownProperty>>, NonNegativeInteger),
	MaxCardinality(Option<TId<UnknownProperty>>, NonNegativeInteger),
	Cardinality(Option<TId<UnknownProperty>>, NonNegativeInteger),
}

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::OnProperty(p, _) => Property::OnProperty(*p),
			Self::SomeValuesFrom(p, _) => Property::SomeValuesFrom(*p),
			Self::AllValuesFrom(p, _) => Property::AllValuesFrom(*p),
			Self::MinCardinality(p, _) => Property::MinCardinality(*p),
			Self::MaxCardinality(p, _) => Property::MaxCardinality(*p),
			Self::Cardinality(p, _) => Property::Cardinality(*p),
		}
	}

	pub fn value(&self) -> BindingValueRef {
		match self {
			Self::OnProperty(_, v) => BindingValueRef::Id(*v),
			Self::SomeValuesFrom(_, v) => BindingValueRef::Id(*v),
			Self::AllValuesFrom(_, v) => BindingValueRef::Id(*v),
			Self::MinCardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
			Self::MaxCardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
			Self::Cardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
		}
	}
}

#[derive(Debug)]
pub enum ClassBindingRef<'a> {
	OnProperty(Option<TId<UnknownProperty>>, Id),
	SomeValuesFrom(Option<TId<UnknownProperty>>, Id),
	AllValuesFrom(Option<TId<UnknownProperty>>, Id),
	MinCardinality(Option<TId<UnknownProperty>>, &'a NonNegativeInteger),
	MaxCardinality(Option<TId<UnknownProperty>>, &'a NonNegativeInteger),
	Cardinality(Option<TId<UnknownProperty>>, &'a NonNegativeInteger),
}

impl<'a> ClassBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::OnProperty(p, _) => Property::OnProperty(*p),
			Self::SomeValuesFrom(p, _) => Property::SomeValuesFrom(*p),
			Self::AllValuesFrom(p, _) => Property::AllValuesFrom(*p),
			Self::MinCardinality(p, _) => Property::MinCardinality(*p),
			Self::MaxCardinality(p, _) => Property::MaxCardinality(*p),
			Self::Cardinality(p, _) => Property::Cardinality(*p),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a> {
		match self {
			Self::OnProperty(_, v) => BindingValueRef::Id(*v),
			Self::SomeValuesFrom(_, v) => BindingValueRef::Id(*v),
			Self::AllValuesFrom(_, v) => BindingValueRef::Id(*v),
			Self::MinCardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
			Self::MaxCardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
			Self::Cardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
		}
	}
}

pub type Binding = ClassBinding;

pub type BindingRef<'a> = ClassBindingRef<'a>;

pub struct ClassBindings<'a, M> {
	on_property: functional_property_value::Iter<'a, Id, M>,
	restriction: single::Iter<'a, Restriction, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_property
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBindingRef::OnProperty))
			.or_else(|| {
				self.restriction
					.next()
					.map(|m| m.map(Restriction::as_binding_ref))
			})
	}
}
