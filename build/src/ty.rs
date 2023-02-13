use crate::{
	context::{HasType, MapIds, MapIdsIn},
	functional_property_value, property_values, rdf,
	resource::{self, BindingValueRef},
	Context, Error, FunctionalPropertyValue, ObjectAsRequiredId, PropertyValue, PropertyValueRef,
	PropertyValues,
};
use locspan::Meta;
use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet},
};
use treeldr::{
	metadata::Merge, prop::UnknownProperty, utils::SccGraph, vocab::Object, Id, Multiple, TId,
};

pub mod datatype;
pub mod restriction;

pub use restriction::{Cardinality, Range, Restriction};
pub use treeldr::ty::{Kind, Property, SubClass, Type};

#[derive(Clone)]
pub struct Data<M> {
	/// Super classes.
	sub_class_of: PropertyValues<crate::Type, M>,

	/// Union.
	union_of: FunctionalPropertyValue<Id, M>,

	/// Intersection.
	intersection_of: FunctionalPropertyValue<Id, M>,

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
		self.union_of
			.map_ids_in(Some(Property::UnionOf(None).into()), &f);
		self.intersection_of
			.map_ids_in(Some(Property::IntersectionOf(None).into()), &f);
		self.properties.map_ids(f)
	}
}

impl<M> Default for Data<M> {
	fn default() -> Self {
		Self {
			sub_class_of: PropertyValues::default(),
			union_of: FunctionalPropertyValue::default(),
			intersection_of: FunctionalPropertyValue::default(),
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

	pub fn sub_class_of(&self) -> &PropertyValues<crate::Type, M> {
		&self.data.sub_class_of
	}

	pub fn sub_class_of_mut(&mut self) -> &mut PropertyValues<crate::Type, M> {
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
			for PropertyValueRef {
				value: Meta(super_class, _),
				..
			} in &self.data.sub_class_of
			{
				if context.is_subclass_of_with(visited, other, *super_class) {
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
		for PropertyValue {
			value: Meta(ty, meta),
			..
		} in self.super_classes(context, as_resource)
		{
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
			Property::SubClassOf(p) => {
				self.sub_class_of_mut()
					.insert(p, prop_cmp, rdf::from::expect_type(value)?)
			}
			Property::UnionOf(p) => {
				self.union_of_mut()
					.insert(p, prop_cmp, rdf::from::expect_id(value)?)
			}
			Property::IntersectionOf(p) => {
				self.intersection_of_mut()
					.insert(p, prop_cmp, rdf::from::expect_id(value)?)
			}
			Property::Datatype(prop) => self.as_datatype_mut().set(prop_cmp, prop, value)?,
			Property::Restriction(prop) => self.as_restriction_mut().set(prop_cmp, prop, value)?,
		}

		Ok(())
	}
}

pub struct SuperClasses<'a, M> {
	resource: Option<&'a M>,
	literal: Option<&'a M>,
	sub_class_of: property_values::non_functional::Iter<'a, crate::Type, M>,
}

impl<'a, M> Iterator for SuperClasses<'a, M> {
	type Item = PropertyValue<crate::Type, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.resource
			.take()
			.map(|meta| PropertyValue::new(None, Meta(crate::Type::Resource(None), meta)))
			.or_else(|| {
				self.literal
					.take()
					.map(|meta| {
						PropertyValue::new(
							None,
							Meta(crate::Type::Resource(Some(resource::Type::Literal)), meta),
						)
					})
					.or_else(|| {
						self.sub_class_of
							.next()
							.map(PropertyValueRef::into_cloned_value)
					})
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
	pub fn union_of(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.data.union_of
	}

	pub fn union_of_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.data.union_of
	}

	pub fn intersection_of(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.data.intersection_of
	}

	pub fn intersection_of_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
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
			Property::UnionOf(None),
		)?;
		let intersection_of = self
			.data
			.intersection_of
			.clone()
			.into_list_at_node_binding(context, as_resource.id, Property::IntersectionOf(None))?;

		let sub_class_of = self
			.data
			.sub_class_of
			.try_mapped(|_, Meta(ty, m)| {
				context
					.require_type_id(ty.raw_id())
					.map(|ty| Meta(ty, m.clone()))
			})
			.map_err(|(Meta(e, m), _)| {
				e.at_node_property(as_resource.id, Property::SubClassOf(None), m.clone())
			})?;

		let desc = if let Some(m) = as_resource.type_metadata(context, SubClass::DataType) {
			treeldr::ty::Description::Data(self.datatype.build(context, as_resource, m)?)
		} else if let Some(m) = as_resource.type_metadata(context, SubClass::Restriction) {
			treeldr::ty::Description::Restriction(self.restriction.build(
				context,
				as_resource,
				m,
			)?)
		} else if let Some(union_of) = union_of.as_required() {
			let mut options = Multiple::default();

			for item in union_of.value().iter(context) {
				let Meta(object, option_causes) = item?.cloned();
				let option_id = object.into_required_id(&option_causes)?;
				let option_ty = context
					.require_type_id(option_id)
					.map_err(|e| e.at(option_causes.clone()))?;

				options.insert(Meta(option_ty, option_causes))
			}

			treeldr::ty::Description::Union(treeldr::ty::Union::new(
				treeldr::RequiredFunctionalPropertyValue::new(
					union_of.sub_properties().clone(),
					options,
				),
			))
		} else if let Some(intersection_of) = intersection_of.as_required() {
			let mut factors = Multiple::default();

			for item in intersection_of.value().iter(context) {
				let Meta(object, factor_causes) = item?.cloned();
				let factor_id = object.into_required_id(&factor_causes)?;
				let factor_ty = context
					.require_type_id(factor_id)
					.map_err(|e| e.at(factor_causes.clone()))?;
				factors.insert(Meta(factor_ty, factor_causes))
			}

			let desc =
				match treeldr::ty::Intersection::new(treeldr::RequiredFunctionalPropertyValue::new(
					intersection_of.sub_properties().clone(),
					factors,
				)) {
					Ok(intersection) => treeldr::ty::Description::Intersection(intersection),
					Err(_) => treeldr::ty::Description::Empty,
				};

			desc
		} else {
			let result = treeldr::ty::Normal::new(sub_class_of);
			treeldr::ty::Description::Normal(result)
		};

		Ok(Meta(treeldr::ty::Definition::new(desc), meta))
	}
}

pub enum ClassBinding {
	UnionOf(Option<TId<UnknownProperty>>, Id),
	IntersectionOf(Option<TId<UnknownProperty>>, Id),
}

impl ClassBinding {
	pub fn as_binding_ref<'a>(&self) -> BindingRef<'a> {
		match self {
			Self::UnionOf(p, i) => BindingRef::UnionOf(*p, *i),
			Self::IntersectionOf(p, i) => BindingRef::IntersectionOf(*p, *i),
		}
	}

	pub fn into_binding_ref<'a>(self) -> BindingRef<'a> {
		match self {
			Self::UnionOf(p, i) => BindingRef::UnionOf(p, i),
			Self::IntersectionOf(p, i) => BindingRef::IntersectionOf(p, i),
		}
	}
}

pub struct ClassBindings<'a, M> {
	union_of: functional_property_value::Iter<'a, Id, M>,
	intersection_of: functional_property_value::Iter<'a, Id, M>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.union_of
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::UnionOf))
			.or_else(|| {
				self.intersection_of
					.next()
					.map(|m| m.into_cloned_class_binding(ClassBinding::IntersectionOf))
			})
	}
}

pub enum Binding {
	UnionOf(Option<TId<UnknownProperty>>, Id),
	IntersectionOf(Option<TId<UnknownProperty>>, Id),
	Datatype(datatype::Binding),
	Restriction(restriction::Binding),
}

impl Binding {
	pub fn property(&self) -> Property {
		match self {
			Self::UnionOf(p, _) => Property::UnionOf(*p),
			Self::IntersectionOf(p, _) => Property::IntersectionOf(*p),
			Self::Datatype(b) => Property::Datatype(b.property()),
			Self::Restriction(b) => Property::Restriction(b.property()),
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<M> {
		match self {
			Self::UnionOf(_, v) => BindingValueRef::Id(*v),
			Self::IntersectionOf(_, v) => BindingValueRef::Id(*v),
			Self::Datatype(b) => b.value(),
			Self::Restriction(b) => b.value(),
		}
	}
}

#[derive(Debug)]
pub enum BindingRef<'a> {
	UnionOf(Option<TId<UnknownProperty>>, Id),
	IntersectionOf(Option<TId<UnknownProperty>>, Id),
	Datatype(datatype::Binding),
	Restriction(restriction::BindingRef<'a>),
}

impl<'a> BindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::UnionOf(p, _) => Property::UnionOf(*p),
			Self::IntersectionOf(p, _) => Property::IntersectionOf(*p),
			Self::Datatype(b) => Property::Datatype(b.property()),
			Self::Restriction(b) => Property::Restriction(b.property()),
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::UnionOf(_, v) => BindingValueRef::Id(*v),
			Self::IntersectionOf(_, v) => BindingValueRef::Id(*v),
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
	type Item = Meta<BindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBinding::into_binding_ref))
			.or_else(|| {
				self.datatype
					.next()
					.map(|m| m.map(BindingRef::Datatype))
					.or_else(|| {
						self.restriction
							.next()
							.map(|m| m.map(BindingRef::Restriction))
					})
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
			let super_classes: HashSet<_> = node
				.as_type()
				.super_classes(context, node.as_resource())
				.map(PropertyValue::into_value)
				.map(crate::Type::into_raw_id)
				.collect();

			// Detect cycles of size 1.
			if super_classes.contains(&id) {
				for PropertyValue {
					value: Meta(i, meta),
					..
				} in node.as_type().super_classes(context, node.as_resource())
				{
					if i.into_raw_id() == id {
						return Err(Meta(
							SubClassCycle(node.id(), Vec::new(), meta.clone()),
							node.metadata().clone(),
						));
					}
				}
			}

			graph.map.insert(id, super_classes);
		}

		let components = graph.strongly_connected_components();

		for component in components.iter() {
			// Detect cycles greater than 1.
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
					.find_map(
						|PropertyValue {
						     value: Meta(ty, m), ..
						 }| {
							if ty.raw_id() == super_id {
								Some(m)
							} else {
								None
							}
						},
					)
					.unwrap();

				super_classes.insert_unique(Meta(crate::Type::from(super_id), meta.clone()));
			}

			map.insert(id, super_classes);
		}

		Ok(Self { map })
	}

	pub fn super_classes(&self, id: Id) -> Option<&Multiple<crate::Type, M>> {
		self.map.get(&id)
	}

	fn remove_indirect_classes_from(&self, result: &mut PropertyValues<crate::Type, M>, id: Id) {
		for super_class in self.super_classes(id).unwrap() {
			result.remove(*super_class);
			self.remove_indirect_classes_from(result, super_class.raw_id());
		}
	}

	pub fn remove_indirect_classes(&self, result: &mut PropertyValues<crate::Type, M>)
	where
		M: Clone,
	{
		let types = result.clone();

		for ty in types {
			self.remove_indirect_classes_from(result, ty.value.raw_id())
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

				node.as_type_mut()
					.sub_class_of_mut()
					.retain(|ty| super_classes.contains(ty));
				for Meta(ty, m) in super_classes {
					if !node.as_type_mut().sub_class_of_mut().contains(&ty) {
						node.as_type_mut()
							.sub_class_of_mut()
							.insert_base_unique(Meta(ty, m));
					}
				}
			}
		}
	}
}
