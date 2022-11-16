use crate::{Error, node, Single, single, resource, Context};
use locspan::Meta;
use treeldr::{metadata::Merge, Id, prop::restriction::Cardinality};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RangeRestriction {
	Any(Id),
	All(Id),
}

pub type CardinalityRestriction = treeldr::prop::restriction::Cardinality;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyRestriction {
	Range(RangeRestriction),
	Cardinality(CardinalityRestriction),
}

#[derive(Clone)]
pub struct Restriction<M> {
	property: Single<Id, M>,
	restriction: Single<PropertyRestriction, M>,
}

impl<M> Default for Restriction<M> {
	fn default() -> Self {
		Self {
			property: Single::default(),
			restriction: Single::default()
		}
	}
}

impl<M> Restriction<M> {
	pub fn property(&self) -> &Single<Id, M> {
		&self.property
	}

	pub fn property_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.property
	}

	pub fn restriction(&self) -> &Single<PropertyRestriction, M> {
		&self.restriction
	}

	pub fn restriction_mut(&mut self) -> &mut Single<PropertyRestriction, M> {
		&mut self.restriction
	}

	pub fn build(
		self,
		context: &Context<M>,
		as_resource: &resource::Data<M>,
		as_class: &super::Data<M>,
		meta: &M
	) -> Result<treeldr::ty::Description<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let prop_ref = self.property.into_required_property_at_node_binding(context, as_resource.id, node::property::Restriction::OnProperty, meta)?;
		let restriction = self.restriction.try_unwrap().map_err(|_| todo!())?.ok_or_else(|| todo!())?;

		let result =
			treeldr::ty::Restriction::new(prop_ref, restriction.build(context, as_resource.id, meta)?);
		Ok(treeldr::ty::Description::Restriction(result))
	}
}

impl PropertyRestriction {
	pub fn build<M>(
		self,
		context: &Context<M>,
		id: Id,
		causes: &M,
	) -> Result<treeldr::prop::Restriction, Error<M>>
	where
		M: Clone,
	{
		match self {
			Self::Range(r) => Ok(treeldr::prop::Restriction::Range(r.build(context, id, causes)?)),
			Self::Cardinality(c) => Ok(treeldr::prop::Restriction::Cardinality(c)),
		}
	}

	pub fn as_binding<'a, M>(&'a self, meta: &'a M) -> BindingRef<'a, M> {
		match self {
			Self::Range(r) => r.as_binding(meta),
			Self::Cardinality(r) => {
				match r {
					Cardinality::AtLeast(v) => BindingRef::MinCardinality(Meta(*v, meta)),
					Cardinality::AtMost(v) => BindingRef::MaxCardinality(Meta(*v, meta)),
					Cardinality::Exactly(v) => BindingRef::Cardinality(Meta(*v, meta))
				}
			}
		}
	}
}

impl RangeRestriction {
	pub fn build<M>(
		self,
		context: &Context<M>,
		id: Id,
		meta: &M,
	) -> Result<treeldr::prop::restriction::Range, Error<M>>
	where
		M: Clone,
	{
		match self {
			Self::Any(ty_id) => {
				let ty_ref = context.require_type(ty_id).map_err(|e| e.at_node_property(id, node::property::Restriction::SomeValuesFrom, meta.clone()))?;
				Ok(treeldr::prop::restriction::Range::Any(**ty_ref))
			}
			Self::All(ty_id) => {
				let ty_ref = context.require_type(ty_id).map_err(|e| e.at_node_property(id, node::property::Restriction::AllValuesFrom, meta.clone()))?;
				Ok(treeldr::prop::restriction::Range::All(**ty_ref))
			}
		}
	}

	pub fn as_binding<'a, M>(&'a self, meta: &'a M) -> BindingRef<'a, M> {
		match self {
			Self::Any(v) => BindingRef::SomeValuesFrom(Meta(*v, meta)),
			Self::All(v) => BindingRef::AllValuesFrom(Meta(*v, meta))
		}
	}
}

pub enum BindingRef<'a, M> {
	OnProperty(Meta<Id, &'a M>),
	SomeValuesFrom(Meta<Id, &'a M>),
	AllValuesFrom(Meta<Id, &'a M>),
	MinCardinality(Meta<u32, &'a M>),
	MaxCardinality(Meta<u32, &'a M>),
	Cardinality(Meta<u32, &'a M>)
}

pub struct Bindings<'a, M> {
	on_property: single::Iter<'a, Id, M>,
	restriction: single::Iter<'a, PropertyRestriction, M>
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = BindingRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_property
			.next()
			.map(Meta::into_cloned_value)
			.map(BindingRef::OnProperty)
			.or_else(|| {
				self.restriction
					.next()
					.map(|Meta(r, meta)| r.as_binding(meta))
			})
	}
}