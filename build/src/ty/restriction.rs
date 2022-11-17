use crate::{Error, Single, single, Context};
use locspan::Meta;
use treeldr::{metadata::Merge, Id};

pub use treeldr::ty::restriction::{Property, Cardinality};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Range {
	Any(Id),
	All(Id),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Restriction {
	Range(Range),
	Cardinality(Cardinality),
}

#[derive(Clone)]
pub struct Definition<M> {
	property: Single<Id, M>,
	restriction: Single<Restriction, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			property: Single::default(),
			restriction: Single::default()
		}
	}
}

impl<M> Definition<M> {
	pub fn property(&self) -> &Single<Id, M> {
		&self.property
	}

	pub fn property_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.property
	}

	pub fn restriction(&self) -> &Single<Restriction, M> {
		&self.restriction
	}

	pub fn restriction_mut(&mut self) -> &mut Single<Restriction, M> {
		&mut self.restriction
	}

	pub fn build(
		self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		meta: &M
	) -> Result<treeldr::ty::restriction::Definition<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let prop_ref = self.property.into_required_property_at_node_binding(context, as_resource.id, Property::OnProperty, &meta)?;
		let restriction = self.restriction.try_unwrap().map_err(|_| todo!())?.ok_or_else(|| todo!())?;

		Ok(treeldr::ty::restriction::Definition::new(prop_ref, restriction.build(context, as_resource.id, meta.clone())?))
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
			Self::Cardinality(c) => treeldr::ty::Restriction::Cardinality(c)
		};

		Ok(Meta(r, meta))
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
				let ty_ref = context.require_type_id(ty_id).map_err(|e| e.at_node_property(id, Property::SomeValuesFrom, meta.clone()))?;
				Ok(treeldr::ty::restriction::Range::Any(ty_ref))
			}
			Self::All(ty_id) => {
				let ty_ref = context.require_type_id(ty_id).map_err(|e| e.at_node_property(id, Property::AllValuesFrom, meta.clone()))?;
				Ok(treeldr::ty::restriction::Range::All(ty_ref))
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
	restriction: single::Iter<'a, Restriction, M>
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