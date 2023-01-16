use crate::{
	context::{HasType, MapIds, MapIdsIn},
	resource::{self, BindingValueRef},
	single, Context, Error, ObjectAsRequiredId, Single,
};
use locspan::Meta;
use std::collections::{HashMap, HashSet};
use treeldr::{metadata::Merge, multiple, utils::SccGraph, Id, Multiple};

pub mod datatype;
pub mod restriction;

pub use restriction::{Cardinality, Range, Restriction};
pub use treeldr::ty::{Kind, Property, SubClass, Type};

#[derive(Clone)]
pub struct Data<M> {
	/// Super classes.
	sub_class_of: Multiple<crate::Type, M>,

	/// Union.
	union_of: Single<Id, M>,

	/// Intersection.
	intersection_of: Single<Id, M>,

	/// Properties.
	properties: HashMap<Id, M>,
}

impl<M> Data<M> {
	pub fn bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			union_of: self.union_of.iter(),
			intersection_of: self.intersection_of.iter(),
		}
	}
}

impl<M: Merge> MapIds for Data<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.union_of.map_ids_in(Some(Property::UnionOf.into()), &f);
		self.intersection_of
			.map_ids_in(Some(Property::IntersectionOf.into()), &f);
		self.properties.map_ids(f)
	}
}

impl<M> Default for Data<M> {
	fn default() -> Self {
		Self {
			sub_class_of: Multiple::default(),
			union_of: Single::default(),
			intersection_of: Single::default(),
			properties: HashMap::new(),
		}
	}
}

#[derive(Clone)]
pub struct Definition<M> {
	data: Data<M>,

	/// Datatype.
	datatype: datatype::Definition<M>,

	/// Restriction.
	restriction: restriction::Definition<M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			data: Data::default(),
			datatype: datatype::Definition::default(),
			restriction: restriction::Definition::default(),
		}
	}
}

impl<M> Definition<M> {
	/// Create a new type.
	///
	/// By default, a normal type is created.
	/// It can later be changed into a non-normal type as long as no properties
	/// have been defined on it.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn sub_class_of(&self) -> &Multiple<crate::Type, M> {
		&self.data.sub_class_of
	}

	pub fn sub_class_of_mut(&mut self) -> &mut Multiple<crate::Type, M> {
		&mut self.data.sub_class_of
	}

	pub fn is_subclass_of_with(
		&self,
		context: &Context<M>,
		visited: &mut HashSet<crate::Type>,
		as_resource: &resource::Data<M>,
		other: crate::Type,
	) -> bool {
		match other {
			crate::Type::Resource(None)
				if as_resource.id != crate::Type::Resource(None).raw_id() =>
			{
				return true
			}
			crate::Type::Resource(Some(resource::Type::Literal)) => {
				if as_resource.has_type(context, SubClass::DataType) {
					return true;
				}
			}
			_ => (),
		}

		if self.data.sub_class_of.contains(&other) {
			true
		} else {
			for Meta(super_class, _) in &self.data.sub_class_of {
				if context.is_subclass_of_with(visited, *super_class, other) {
					return true;
				}
			}

			false
		}
	}

	pub fn super_classes<'a>(
		&'a self,
		context: &Context<M>,
		as_resource: &'a resource::Data<M>,
	) -> SuperClasses<'a, M> {
		SuperClasses {
			resource: if as_resource.id == crate::Type::Resource(None).raw_id()
				|| self
					.data
					.sub_class_of
					.contains(&crate::Type::Resource(None))
			{
				None
			} else {
				Some(&as_resource.metadata)
			},
			literal: if self
				.data
				.sub_class_of
				.contains(&crate::Type::Resource(Some(resource::Type::Literal)))
			{
				None
			} else {
				as_resource.type_metadata(context, SubClass::DataType)
			},
			sub_class_of: self.data.sub_class_of.iter(),
		}
	}

	fn find_sub_class_cycle_from(
		&self,
		context: &Context<M>,
		as_resource: &resource::Data<M>,
		component: &[Id],
		visited: &HashSet<Id>,
	) -> Option<(Vec<Meta<Id, M>>, M)>
	where
		M: Clone,
	{
		for Meta(ty, meta) in self.super_classes(context, as_resource) {
			let id = ty.raw_id();
			if id == component[0] {
				return Some((Vec::new(), meta.clone()));
			}

			if !visited.contains(&id) && component.contains(&id) {
				let mut visited = visited.clone();
				visited.insert(id);
				let node = context.get(id).unwrap();
				if let Some((mut path, end)) = node.as_type().find_sub_class_cycle_from(
					context,
					node.as_resource(),
					component,
					&visited,
				) {
					path.push(Meta(id, meta.clone()));
					return Some((path, end));
				}
			}
		}

		None
	}

	/// Find a path to the first member of the given super-class `component`.
	///
	/// This computes a path from this node to the first member of `component`
	/// by only going through super classes contained in `component`.
	///
	/// The path is given in reverse order (for performance reasons).
	/// The last segment contains only the metadata.
	pub fn find_sub_class_cycle(
		&self,
		context: &Context<M>,
		as_resource: &resource::Data<M>,
		component: &[Id],
	) -> Option<(Vec<Meta<Id, M>>, M)>
	where
		M: Clone,
	{
		self.find_sub_class_cycle_from(context, as_resource, component, &HashSet::new())
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.data.bindings(),
			datatype: self.datatype.bindings(),
			restriction: self.restriction.bindings(),
		}
	}
}

pub struct SuperClasses<'a, M> {
	resource: Option<&'a M>,
	literal: Option<&'a M>,
	sub_class_of: multiple::Iter<'a, crate::Type, M>,
}

impl<'a, M> Iterator for SuperClasses<'a, M> {
	type Item = Meta<crate::Type, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.resource
			.take()
			.map(|meta| Meta(crate::Type::Resource(None), meta))
			.or_else(|| {
				self.literal
					.take()
					.map(|meta| Meta(crate::Type::Resource(Some(resource::Type::Literal)), meta))
					.or_else(|| self.sub_class_of.next().map(Meta::into_cloned_value))
			})
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
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
	) -> Result<Meta<treeldr::ty::Definition<M>, M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let union_of = self.data.union_of.clone().into_list_at_node_binding(
			context,
			as_resource.id,
			Property::UnionOf,
		)?;
		let intersection_of = self
			.data
			.intersection_of
			.clone()
			.into_list_at_node_binding(context, as_resource.id, Property::IntersectionOf)?;

		let mut sub_class_of = Multiple::default();
		for Meta(ty, m) in &self.data.sub_class_of {
			let id = context
				.require_type_id(ty.raw_id())
				.map_err(|e| e.at_node_property(as_resource.id, Property::SubClassOf, m.clone()))?;
			sub_class_of.insert(Meta(id, m.clone()));
		}

		let desc = if let Some(m) = as_resource.type_metadata(context, SubClass::DataType) {
			Meta(
				treeldr::ty::Description::Data(self.datatype.build(context, as_resource, &meta)?),
				m.clone(),
			)
		} else if let Some(m) = as_resource.type_metadata(context, SubClass::Restriction) {
			Meta(
				treeldr::ty::Description::Restriction(self.restriction.build(
					context,
					as_resource,
					&meta,
				)?),
				m.clone(),
			)
		} else if let Some(union_of) = union_of.as_ref() {
			let mut options = Multiple::default();

			for item in union_of.iter(context) {
				let Meta(object, option_causes) = item?.cloned();
				let option_id = object.into_required_id(&option_causes)?;
				let option_ty = context
					.require_type_id(option_id)
					.map_err(|e| e.at(option_causes.clone()))?;

				options.insert(Meta(option_ty, option_causes))
			}

			Meta(
				treeldr::ty::Description::Union(treeldr::ty::Union::new(options)),
				union_of.metadata().clone(),
			)
		} else if let Some(intersection_of) = intersection_of.as_ref() {
			let mut factors = Multiple::default();

			for item in intersection_of.iter(context) {
				let Meta(object, factor_causes) = item?.cloned();
				let factor_id = object.into_required_id(&factor_causes)?;
				let factor_ty = context
					.require_type_id(factor_id)
					.map_err(|e| e.at(factor_causes.clone()))?;
				factors.insert(Meta(factor_ty, factor_causes))
			}

			let desc = match treeldr::ty::Intersection::new(factors) {
				Ok(intersection) => treeldr::ty::Description::Intersection(intersection),
				Err(_) => treeldr::ty::Description::Empty,
			};

			Meta(desc, intersection_of.metadata().clone())
		} else {
			let result = treeldr::ty::Normal::new(sub_class_of);
			Meta(treeldr::ty::Description::Normal(result), meta.clone())
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
			Self::IntersectionOf(i) => Binding::IntersectionOf(i),
		}
	}
}

pub struct ClassBindings<'a, M> {
	union_of: single::Iter<'a, Id, M>,
	intersection_of: single::Iter<'a, Id, M>,
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
			Self::Restriction(b) => Property::Restriction(b.property()),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::UnionOf(v) => BindingValueRef::Id(*v),
			Self::IntersectionOf(v) => BindingValueRef::Id(*v),
			Self::Datatype(b) => b.value(),
			Self::Restriction(b) => b.value(),
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
					.or_else(|| self.restriction.next().map(|m| m.map(Binding::Restriction)))
			})
	}
}

#[derive(Debug)]
pub struct SubClassCycle<M>(pub Id, pub Vec<Meta<Id, M>>, pub M);

/// Minimal class hierarchy.
pub struct ClassHierarchy<M> {
	map: HashMap<Id, Multiple<crate::Type, M>>,
}

impl<M> ClassHierarchy<M> {
	pub fn new(context: &Context<M>) -> Result<Self, Meta<SubClassCycle<M>, M>>
	where
		M: Clone,
	{
		struct Graph {
			map: HashMap<Id, HashSet<Id>>,
		}

		impl SccGraph for Graph {
			type Vertex = Id;

			type Vertices<'a> = std::iter::Copied<std::collections::hash_map::Keys<'a, Id, HashSet<Id>>> where Self: 'a;

			type Successors<'a> = std::iter::Copied<std::collections::hash_set::Iter<'a, Id>> where Self: 'a;

			fn vertices(&self) -> Self::Vertices<'_> {
				self.map.keys().copied()
			}

			fn successors(&self, v: Self::Vertex) -> Self::Successors<'_> {
				self.map[&v].iter().copied()
			}
		}

		let mut graph = Graph {
			map: HashMap::new(),
		};

		for (id, node) in context.nodes() {
			let super_classes = node
				.as_type()
				.super_classes(context, node.as_resource())
				.map(Meta::into_value)
				.map(crate::Type::into_raw_id)
				.collect();

			graph.map.insert(id, super_classes);
		}

		let components = graph.strongly_connected_components();

		for component in components.iter() {
			if component.len() > 1 {
				let node = context.get(component[0]).unwrap();
				let (mut path, end) = node
					.as_type()
					.find_sub_class_cycle(context, node.as_resource(), component)
					.unwrap();
				path.reverse();
				return Err(Meta(
					SubClassCycle(node.id(), path, end),
					node.metadata().clone(),
				));
			}
		}

		let mut map = HashMap::new();

		for (i, component) in components.iter().enumerate() {
			let id = component[0];
			let node = context.get(id).unwrap();

			let mut super_classes = Multiple::default();
			for super_i in components.direct_successors(i).unwrap() {
				let super_id = components.get(super_i).unwrap()[0];
				let meta = node
					.as_type()
					.super_classes(context, node.as_resource())
					.find_map(|Meta(ty, m)| {
						if ty.raw_id() == super_id {
							Some(m)
						} else {
							None
						}
					})
					.unwrap();

				super_classes.insert_unique(Meta(crate::Type::from(super_id), meta.clone()));
			}

			map.insert(id, super_classes);
		}

		Ok(Self { map })
	}

	pub fn super_classes(&self, id: Id) -> &Multiple<crate::Type, M> {
		self.map.get(&id).unwrap()
	}

	fn remove_indirect_classes_from(&self, result: &mut Multiple<crate::Type, M>, id: Id) {
		for super_class in self.super_classes(id) {
			result.remove(*super_class);
			self.remove_indirect_classes_from(result, super_class.raw_id());
		}
	}

	pub fn remove_indirect_classes(&self, result: &mut Multiple<crate::Type, M>)
	where
		M: Clone,
	{
		let types = result.clone();

		for ty in types {
			self.remove_indirect_classes_from(result, ty.raw_id())
		}
	}

	pub fn apply(self, context: &mut Context<M>)
	where
		M: Clone,
	{
		for (_, node) in context.nodes_mut() {
			self.remove_indirect_classes(node.type_mut())
		}

		for (id, super_classes) in self.map {
			let node = context.get(id).unwrap();
			if node.has_type(context, resource::Type::Class(None)) {
				let node = context.get_mut(id).unwrap();
				*node.as_type_mut().sub_class_of_mut() = super_classes;
			}
		}
	}
}
