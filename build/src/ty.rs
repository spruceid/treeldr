use crate::{Error, ObjectToId, node, Single, single};
use locspan::Meta;
use std::collections::{BTreeMap, HashMap, btree_map::Entry};
use treeldr::{metadata::Merge, Id, MetaOption};

pub mod datatype;
mod restriction;

pub use datatype::DataType;
pub use restriction::{Restriction, RangeRestriction, CardinalityRestriction};
pub use treeldr::ty::Kind;

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
	datatype: DataType<M>,

	/// Restriction.
	restriction: Restriction<M>
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
			datatype: MetaOption::default(),
			restriction: MetaOption::default()
		}
	}
}

impl<M> Definition<M> {
	/// Declare that this type is a datatype.
	pub fn declare_datatype(&mut self, metadata: M)
	where
		M: Merge,
	{
		self.datatype.set_once(metadata, || DataType::default())
	}

	/// Declare that this type is a restriction.
	pub fn declare_restriction(&mut self, metadata: M)
	where
		M: Merge,
	{
		self.restriction.set_once(metadata, || Restriction::default())
	}

	pub fn declare_union(&mut self, list_id: Meta<Id, M>) where M: Merge {
		self.union_of.insert(list_id)
	}

	pub fn declare_intersection(&mut self, list_id: Meta<Id, M>) where M: Merge {
		self.intersection_of.insert(list_id)
	}

	/// Declare a property of the type.
	pub fn declare_property(&mut self, prop_ref: Id, cause: M)
	where
		M: Merge,
	{
		use std::collections::hash_map::Entry;
		match self.properties.entry(prop_ref) {
			Entry::Vacant(entry) => {
				entry.insert(cause);
			}
			Entry::Occupied(mut entry) => entry.get_mut().merge_with(cause),
		}
	}
}

impl<M: Clone> Definition<M> {
	pub fn dependencies(
		&self,
		nodes: &super::context::allocated::Nodes<M>,
		id: Id,
		_causes: &M,
	) -> Result<Vec<crate::Item<M>>, Error<M>> {
		let mut dependencies = Vec::new();
		
		let union_of = self.union_of.clone().into_list_at_node_binding(nodes, id, node::property::Class::UnionOf)?;
		let intersection_of = self.intersection_of.clone().into_list_at_node_binding(nodes, id, node::property::Class::IntersectionOf)?;

		if let Some(union_of) = union_of.as_ref() {
			for item in union_of.iter(nodes) {
				let Meta(object, causes) = item?.cloned();
				let option_id = object.into_required_id(&causes)?;

				let Meta(option_ty, _) =
					nodes.require_type(option_id).map_err(|e| e.at(causes.clone()))?.clone();

				dependencies.push(crate::Item::Type(option_ty))
			}
		}
		
		if let Some(intersection_of) = intersection_of.as_ref() {
			for item in intersection_of.iter(nodes) {
				let Meta(object, causes) = item?.cloned();
				let factor_id = object.into_required_id(&causes)?;

				let Meta(factor_ty, _) =
					nodes.require_type(factor_id).map_err(|e| e.at(causes.clone()))?.clone();

				dependencies.push(crate::Item::Type(factor_ty))
			}
		}

		Ok(dependencies)
	}
}

impl<M: Clone + Merge> crate::Build<M> for Definition<M> {
	type Target = treeldr::ty::Definition<M>;

	fn build(
		self,
		nodes: &mut super::context::allocated::Nodes<M>,
		dependencies: crate::Dependencies<M>,
		id: Id,
		meta: M,
	) -> Result<Self::Target, Error<M>> {
		let union_of = self.union_of.into_list_at_node_binding(nodes, id, node::property::Class::UnionOf)?;
		let intersection_of = self.intersection_of.into_list_at_node_binding(nodes, id, node::property::Class::IntersectionOf)?;

		let desc = if let Some(Meta(datatype, _)) = self.datatype.unwrap() {
			datatype.build(nodes, dependencies, id, &meta)?
		} else if let Some(Meta(restriction, _)) = self.restriction.unwrap() {
			restriction.build(nodes, id, &meta)?
		} else if let Some(union_of) = union_of.as_ref() {
			let mut options = BTreeMap::new();

			for item in union_of.iter(nodes) {
				let Meta(object, causes) = item?.cloned();
				let option_id = object.into_required_id(&causes)?;

				let Meta(option_ty, option_causes) =
					nodes.require_type(option_id).map_err(|e| e.at(causes.clone()))?.clone();

				match options.entry(option_ty) {
					Entry::Vacant(entry) => {
						entry.insert(option_causes);
					}
					Entry::Occupied(mut entry) => {
						entry.get_mut().merge_with(option_causes);
					}
				}
			}

			treeldr::ty::Description::Union(treeldr::ty::Union::new(options, |ty_ref| {
				dependencies.ty(ty_ref)
			}))
		} else if let Some(intersection_of) = intersection_of.as_ref() {
			let mut factors = BTreeMap::new();

			for item in intersection_of.iter(nodes) {
				let Meta(object, causes) = item?.cloned();
				let factor_id = object.into_required_id(&causes)?;

				let Meta(factor_ty, option_causes) =
					nodes.require_type(factor_id).map_err(|e| e.at(causes.clone()))?.clone();

				match factors.entry(factor_ty) {
					Entry::Vacant(entry) => {
						entry.insert(option_causes);
					}
					Entry::Occupied(mut entry) => {
						entry.get_mut().merge_with(option_causes);
					}
				}
			}

			match treeldr::ty::Intersection::new(factors, |ty_ref| dependencies.ty(ty_ref)) {
				Ok(intersection) => treeldr::ty::Description::Intersection(intersection),
				Err(_) => treeldr::ty::Description::Empty,
			}
		} else {
			let mut result = treeldr::ty::Normal::new();

			for (prop_id, prop_causes) in self.properties {
				let prop_ref = nodes.require_property(prop_id).map_err(|e| e.at(prop_causes.clone()))?;
				result.insert_property(**prop_ref, prop_causes)
			}

			treeldr::ty::Description::Normal(result)
		};

		Ok(treeldr::ty::Definition::new(id, desc, meta))
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