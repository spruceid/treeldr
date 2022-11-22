use crate::{Error, Single, single, Context, resource::BindingValueRef, context::MapIds};
use locspan::Meta;
use treeldr::{metadata::Merge, Id};

pub use treeldr::ty::restriction::{Property, Cardinality};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Range {
	Any(Id),
	All(Id),
}

impl MapIds for Range {
	fn map_ids(&mut self, f: impl Fn(Id) -> Id) {
		match self {
			Self::Any(id) => id.map_ids(f),
			Self::All(id) => id.map_ids(f)
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Restriction {
	Range(Range),
	Cardinality(Cardinality),
}

impl MapIds for Restriction {
	fn map_ids(&mut self, f: impl Fn(Id) -> Id) {
		if let Self::Range(r) = self {
			r.map_ids(f)
		}
	}
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

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings { on_property: self.property.iter(), restriction: self.restriction.iter() }
	}

	pub fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		meta: &M
	) -> Result<treeldr::ty::restriction::Definition<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let prop_ref = self.property.clone().into_required_property_at_node_binding(context, as_resource.id, Property::OnProperty, &meta)?;
		let restriction = self.restriction.clone().try_unwrap().map_err(|_| todo!())?.ok_or_else(|| todo!())?;

		Ok(treeldr::ty::restriction::Definition::new(prop_ref, restriction.build(context, as_resource.id, meta.clone())?))
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id) -> Id) {
		self.property.map_ids(&f);
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
			Self::Cardinality(c) => treeldr::ty::Restriction::Cardinality(c)
		};

		Ok(Meta(r, meta))
	}

	pub fn as_binding(&self) -> ClassBinding {
		match self {
			Self::Range(r) => r.as_binding(),
			Self::Cardinality(r) => {
				match r {
					Cardinality::AtLeast(v) => ClassBinding::MinCardinality(*v),
					Cardinality::AtMost(v) => ClassBinding::MaxCardinality(*v),
					Cardinality::Exactly(v) => ClassBinding::Cardinality(*v)
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

	pub fn as_binding(&self) -> ClassBinding {
		match self {
			Self::Any(v) => ClassBinding::SomeValuesFrom(*v),
			Self::All(v) => ClassBinding::AllValuesFrom(*v)
		}
	}
}

pub enum ClassBinding {
	OnProperty(Id),
	SomeValuesFrom(Id),
	AllValuesFrom(Id),
	MinCardinality(u64),
	MaxCardinality(u64),
	Cardinality(u64)
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::OnProperty(_) => Property::OnProperty,
			Self::SomeValuesFrom(_) => Property::SomeValuesFrom,
			Self::AllValuesFrom(_) => Property::AllValuesFrom,
			Self::MinCardinality(_) => Property::MinCardinality,
			Self::MaxCardinality(_) => Property::MaxCardinality,
			Self::Cardinality(_) => Property::Cardinality
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::OnProperty(v) => BindingValueRef::Id(*v),
			Self::SomeValuesFrom(v) => BindingValueRef::Id(*v),
			Self::AllValuesFrom(v) => BindingValueRef::Id(*v),
			Self::MinCardinality(v) => BindingValueRef::U64(*v),
			Self::MaxCardinality(v) => BindingValueRef::U64(*v),
			Self::Cardinality(v) => BindingValueRef::U64(*v)
		}
	}
}

pub struct ClassBindings<'a, M> {
	on_property: single::Iter<'a, Id, M>,
	restriction: single::Iter<'a, Restriction, M>
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_property
			.next()
			.map(Meta::into_cloned_value)
			.map(|m| m.map(ClassBinding::OnProperty))
			.or_else(|| {
				self.restriction
					.next()
					.map(|m| m.map(Restriction::as_binding))
			})
	}
}