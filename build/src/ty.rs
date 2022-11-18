use crate::{Error, ObjectAsRequiredId, Single, single, context::HasType};
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

pub enum BindingRef<'a, M> {
	Datatype(datatype::BindingRef<'a, M>),
	Restriction(restriction::BindingRef<'a, M>),
	UnionOf(Meta<Id, &'a M>),
	IntersectionOf(Meta<Id, &'a M>),
}

pub struct Bindings<'a, M> {
	datatype: datatype::Bindings<'a, M>,
	restriction: restriction::Bindings<'a, M>,
	union_of: single::Iter<'a, Id, M>,
	intersection_of: single::Iter<'a, Id, M>
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = BindingRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.datatype
			.next()
			.map(BindingRef::Datatype)
			.or_else(|| {
				self.restriction
					.next()
					.map(BindingRef::Restriction)
					.or_else(|| {
						self.union_of
							.next()
							.map(Meta::into_cloned_value)
							.map(BindingRef::UnionOf)
							.or_else(|| {
								self.intersection_of
									.next()
									.map(Meta::into_cloned_value)
									.map(BindingRef::IntersectionOf)
							})
					})
			})
	}
}