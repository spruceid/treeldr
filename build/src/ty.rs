use crate::{Error, ObjectAsRequiredId, Single, single, context::{HasType, MapIds}, resource::BindingValueRef};
use locspan::Meta;
use std::collections::HashMap;
use treeldr::{metadata::Merge, Id, Multiple};

pub mod datatype;
pub mod restriction;

pub use restriction::{Restriction, Range, Cardinality};
pub use treeldr::ty::{Kind, Type, SubClass, Property};

#[derive(Clone)]
pub struct Data<M> {
	/// Union.
	union_of: Single<Id, M>,

	/// Intersection.
	intersection_of: Single<Id, M>,

	/// Properties.
	properties: HashMap<Id, M>,
}

impl<M> Data<M> {
	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings { union_of: self.union_of.iter(), intersection_of: self.intersection_of.iter() }
	}
}

impl<M: Merge> MapIds for Data<M> {
	fn map_ids(&mut self, f: impl Fn(Id) -> Id) {
		self.union_of.map_ids(&f);
		self.intersection_of.map_ids(&f);
		self.properties.map_ids(f)
	}
}

impl<M> Default for Data<M> {
	fn default() -> Self {
		Self {
			union_of: Single::default(),
			intersection_of: Single::default(),
			properties: HashMap::new()
		}
	}
}

#[derive(Clone)]
pub struct Definition<M> {
	data: Data<M>,

	/// Datatype.
	datatype: datatype::Definition<M>,

	/// Restriction.
	restriction: restriction::Definition<M>
}

impl<M> Definition<M> {
	/// Create a new type.
	///
	/// By default, a normal type is created.
	/// It can later be changed into a non-normal type as long as no properties
	/// have been defined on it.
	pub fn new() -> Self {
		Self {
			data: Data::default(),
			datatype: datatype::Definition::default(),
			restriction: restriction::Definition::default()
		}
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.data.bindings(),
			datatype: self.datatype.bindings(),
			restriction: self.restriction.bindings()
		}
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id) -> Id) {
		self.data.map_ids(&f);
		self.datatype.map_ids(&f);
		self.restriction.map_ids(f)
	}
}

impl<M> Definition<M> {
	pub fn union_of(&self) -> &Single<Id, M> {
		&self.data.union_of
	}

	pub fn union_of_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.data.union_of
	}

	pub fn intersection_of(&self) -> &Single<Id, M> {
		&self.data.intersection_of
	}

	pub fn intersection_of_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.data.intersection_of
	}

	pub fn as_datatype(&self) -> &datatype::Definition<M> {
		&self.datatype
	}

	pub fn as_datatype_mut(&mut self) -> &mut datatype::Definition<M> {
		&mut self.datatype
	}

	pub fn as_restriction(&self) -> &restriction::Definition<M> {
		&self.restriction
	}

	pub fn as_restriction_mut(&mut self) -> &mut restriction::Definition<M> {
		&mut self.restriction
	}

	pub(crate) fn build(
		&self,
		context: &crate::Context<M>,
		as_resource: &treeldr::node::Data<M>,
		meta: M,
	) -> Result<Meta<treeldr::ty::Definition<M>, M>, Error<M>> where M: Clone + Merge {
		let union_of = self.data.union_of.clone().into_list_at_node_binding(context, as_resource.id, Property::UnionOf)?;
		let intersection_of = self.data.intersection_of.clone().into_list_at_node_binding(context, as_resource.id, Property::IntersectionOf)?;

		let desc = if as_resource.has_type(context, SubClass::DataType) {
			treeldr::ty::Description::Data(self.datatype.build(context, as_resource, &meta)?)
		} else if as_resource.has_type(context, SubClass::Restriction) {
			treeldr::ty::Description::Restriction(self.restriction.build(context, as_resource, &meta)?)
		} else if let Some(union_of) = union_of.as_ref() {
			let mut options = Multiple::default();

			for item in union_of.iter(context) {
				let Meta(object, option_causes) = item?.cloned();
				let option_id = object.into_required_id(&option_causes)?;
				let option_ty = context.require_type_id(option_id).map_err(|e| e.at(option_causes.clone()))?;

				options.insert(Meta(option_ty, option_causes))
			}

			treeldr::ty::Description::Union(treeldr::ty::Union::new(options))
		} else if let Some(intersection_of) = intersection_of.as_ref() {
			let mut factors = Multiple::default();

			for item in intersection_of.iter(context) {
				let Meta(object, factor_causes) = item?.cloned();
				let factor_id = object.into_required_id(&factor_causes)?;
				let factor_ty = context.require_type_id(factor_id).map_err(|e| e.at(factor_causes.clone()))?;
				factors.insert(Meta(factor_ty, factor_causes))
			}

			match treeldr::ty::Intersection::new(factors) {
				Ok(intersection) => treeldr::ty::Description::Intersection(intersection),
				Err(_) => treeldr::ty::Description::Empty,
			}
		} else {
			let mut result = treeldr::ty::Normal::new();

			for (prop_id, prop_causes) in &self.data.properties {
				let prop_ref = context.require_property_id(*prop_id).map_err(|e| e.at(prop_causes.clone()))?;
				result.insert_property(prop_ref, prop_causes.clone())
			}

			treeldr::ty::Description::Normal(result)
		};

		Ok(Meta(treeldr::ty::Definition::new(desc), meta))
	}
}

pub enum ClassBinding {
	UnionOf(Id),
	IntersectionOf(Id),
}

impl ClassBinding {
	pub fn into_binding(self) -> Binding {
		match self {
			Self::UnionOf(i) => Binding::UnionOf(i),
			Self::IntersectionOf(i) => Binding::IntersectionOf(i)
		}
	}
}

pub struct ClassBindings<'a, M> {
	union_of: single::Iter<'a, Id, M>,
	intersection_of: single::Iter<'a, Id, M>
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.union_of
			.next()
			.map(Meta::into_cloned_value)
			.map(|m| m.map(ClassBinding::UnionOf))
			.or_else(|| {
				self.intersection_of
					.next()
					.map(Meta::into_cloned_value)
					.map(|m| m.map(ClassBinding::IntersectionOf))
			})
	}
}

pub enum Binding {
	UnionOf(Id),
	IntersectionOf(Id),
	Datatype(datatype::Binding),
	Restriction(restriction::Binding),
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::UnionOf(_) => Property::UnionOf,
			Self::IntersectionOf(_) => Property::IntersectionOf,
			Self::Datatype(b) => Property::Datatype(b.property()),
			Self::Restriction(b) => Property::Restriction(b.property())
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::UnionOf(v) => BindingValueRef::Id(*v),
			Self::IntersectionOf(v) => BindingValueRef::Id(*v),
			Self::Datatype(b) => b.value(),
			Self::Restriction(b) => b.value()
		}
	}
}

pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	datatype: datatype::Bindings<'a, M>,
	restriction: restriction::Bindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<Binding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBinding::into_binding))
			.or_else(|| {
				self.datatype
					.next()
					.map(|m| m.map(Binding::Datatype))
					.or_else(|| {
						self.restriction
							.next()
							.map(|m| m.map(Binding::Restriction))
					})
			})
	}
}