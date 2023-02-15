use std::{
	cell::RefCell,
	cmp::Ordering,
	collections::{HashMap, HashSet},
};

use crate::{
	context::{HasType, MapIds, MapIdsIn},
	functional_property_value, property_values, rdf,
	resource::BindingValueRef,
	Context, Error, FunctionalPropertyValue, PropertyValues,
};
use const_vec::ConstVec;
use locspan::{Meta, Stripped};
use treeldr::{
	metadata::Merge,
	prop::{RdfProperty, UnknownProperty},
	utils::SccGraph,
	vocab::{self, Object},
	Id, Multiple, TId,
};

pub use treeldr::prop::{Property, Type};

/// Property definition.
#[derive(Clone)]
pub struct Definition<M> {
	/// Domain.
	domain: PropertyValues<Id, M>,

	/// Range.
	range: PropertyValues<Id, M>,

	/// Is the property required.
	required: FunctionalPropertyValue<bool, M>,

	/// Super properties.
	sub_property_of: PropertyValues<Id, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			domain: PropertyValues::default(),
			range: PropertyValues::default(),
			required: FunctionalPropertyValue::default(),
			sub_property_of: PropertyValues::default(),
		}
	}
}

#[derive(Debug)]
pub struct SubPropertyCycle<M>(pub Id, pub Vec<Meta<Id, M>>, pub M);

/// Computes the explicitly defined super properties for each properties.
pub struct ExplicitSuperProperties<M> {
	computed: ConstVec<Multiple<Id, M>>,
	indexes: RefCell<HashMap<Id, Option<usize>>>,
}

impl<M> ExplicitSuperProperties<M> {
	const SUB_PROPERTY_OF_ID: Id = Id::Iri(treeldr::IriIndex::Iri(treeldr::vocab::Term::Rdfs(
		treeldr::vocab::Rdfs::SubPropertyOf,
	)));

	pub fn new(context: &crate::Context<M>) -> Self {
		Self {
			computed: ConstVec::new(context.len()),
			indexes: RefCell::new(HashMap::new()),
		}
	}
}

impl<M: Clone + Merge> ExplicitSuperProperties<M> {
	/// Checks if `b` is a sub property of `a`.
	pub fn is_sub_property_of(&self, context: &crate::Context<M>, a: Id, b: Id) -> bool {
		if let Some(super_properties) = self.get_recursive(context, b) {
			for Meta(s, _) in super_properties {
				if *s == a || self.is_sub_property_of(context, a, *s) {
					return true;
				}
			}
		}

		false
	}

	/// Returns the super properties of the property `id`.
	pub fn get(&self, context: &crate::Context<M>, id: Id) -> &Multiple<Id, M> {
		self.get_recursive(context, id).unwrap()
	}

	/// Returns the super properties of the property `id`.
	///
	/// This is the recursive version of `get`, that may return `None` if
	/// recursively called on the same `id`.
	fn get_recursive(&self, context: &crate::Context<M>, id: Id) -> Option<&Multiple<Id, M>> {
		let indexes = self.indexes.borrow();
		let i: usize = match indexes.get(&id) {
			Some(Some(i)) => *i,
			Some(None) => return None,
			None => {
				std::mem::drop(indexes);

				{
					let mut indexes = self.indexes.borrow_mut();
					indexes.insert(id, None);
				}

				let node = context.get(id).unwrap();

				let mut result = Multiple::default();

				for v in node.as_property().sub_property_of() {
					result.insert(v.value.cloned())
				}

				for (prop, values) in node.other_properties() {
					if self.is_sub_property_of(context, Self::SUB_PROPERTY_OF_ID, *prop) {
						for v in values {
							let id = match v.value.cloned().map(Stripped::unwrap) {
								Meta(vocab::Object::Literal(_), _) => None,
								Meta(vocab::Object::Blank(id), meta) => {
									Some(Meta(Id::Blank(id), meta))
								}
								Meta(vocab::Object::Iri(id), meta) => Some(Meta(Id::Iri(id), meta)),
							};

							if let Some(id) = id {
								result.insert(id);
							}
						}
					}
				}

				let i = self.computed.len();
				self.computed.push(result);

				let mut indexes = self.indexes.borrow_mut();
				indexes.insert(id, Some(i));
				i
			}
		};

		Some(&self.computed[i])
	}

	pub fn find_cycle(
		&self,
		context: &crate::Context<M>,
		id: Id,
		component: &[Id],
	) -> Option<(Vec<Meta<Id, M>>, M)> {
		self.find_cycle_from(context, id, component, &HashSet::new())
	}

	fn find_cycle_from(
		&self,
		context: &Context<M>,
		id: Id,
		component: &[Id],
		visited: &HashSet<Id>,
	) -> Option<(Vec<Meta<Id, M>>, M)>
	where
		M: Clone,
	{
		for Meta(super_id, meta) in self.get(context, id) {
			if *super_id == id {
				return Some((Vec::new(), meta.clone()));
			}

			if !visited.contains(super_id) && component.contains(super_id) {
				let mut visited = visited.clone();
				visited.insert(*super_id);
				if let Some((mut path, end)) =
					self.find_cycle_from(context, *super_id, component, &visited)
				{
					path.push(Meta(*super_id, meta.clone()));
					return Some((path, end));
				}
			}
		}

		None
	}
}

pub struct Hierarchy<M> {
	map: HashMap<Id, Multiple<Id, M>>,
}

impl<M> Hierarchy<M> {
	pub fn super_properties(&self, id: Id) -> Option<&Multiple<Id, M>> {
		self.map.get(&id)
	}

	/// Checks if `b` is a sub property of `a`.
	pub fn is_sub_property_of(&self, a: Id, b: Id) -> bool {
		if let Some(super_properties) = self.super_properties(b) {
			for Meta(s, _) in super_properties {
				if *s == a || self.is_sub_property_of(a, *s) {
					return true;
				}
			}
		}

		false
	}

	/// Checks if `b` is a sub property of `a` or equal to `a`.
	pub fn is_sub_property_of_or_eq(&self, a: Id, b: Id) -> bool {
		a == b || self.is_sub_property_of(a, b)
	}

	pub fn cmp(&self, a: Id, b: Id) -> Option<Ordering> {
		if a == b {
			Some(Ordering::Equal)
		} else if self.is_sub_property_of(a, b) {
			Some(Ordering::Greater)
		} else if self.is_sub_property_of(b, a) {
			Some(Ordering::Less)
		} else {
			None
		}
	}
}

impl<M: Clone + Merge> Hierarchy<M> {
	pub fn new(context: &Context<M>) -> Result<Self, Meta<SubPropertyCycle<M>, M>> {
		let all_super_properties = ExplicitSuperProperties::new(context);

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
			let super_properties = all_super_properties.get(context, id);

			// Detect cycles of size 1.
			if let Some(meta) = super_properties.get_metadata(&id) {
				return Err(Meta(
					SubPropertyCycle(node.id(), Vec::new(), meta.clone()),
					node.metadata().clone(),
				));
			}

			graph.map.insert(
				id,
				super_properties
					.into_iter()
					.map(Meta::into_value)
					.cloned()
					.collect(),
			);
		}

		let components = graph.strongly_connected_components();

		for component in components.iter() {
			// Detect cycles greater than 1.
			if component.len() > 1 {
				let node = context.get(component[0]).unwrap();
				let (mut path, end) = all_super_properties
					.find_cycle(context, component[0], component)
					.unwrap();
				path.reverse();
				return Err(Meta(
					SubPropertyCycle(node.id(), path, end),
					node.metadata().clone(),
				));
			}
		}

		let mut map = HashMap::new();

		for (i, component) in components.iter().enumerate() {
			let id = component[0];

			let direct_successors: HashSet<Id> = components
				.direct_successors(i)
				.unwrap()
				.into_iter()
				.map(|j| components.get(j).unwrap()[0])
				.collect();
			let super_properties = all_super_properties
				.get(context, id)
				.iter()
				.map(|Meta(v, m)| Meta(*v, m.clone()))
				.filter(|Meta(super_id, _)| direct_successors.contains(super_id))
				.collect();

			map.insert(id, super_properties);
		}

		Ok(Self { map })
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn range(&self) -> &PropertyValues<Id, M> {
		&self.range
	}

	pub fn range_mut(&mut self) -> &mut PropertyValues<Id, M> {
		&mut self.range
	}

	pub fn domain(&self) -> &PropertyValues<Id, M> {
		&self.domain
	}

	pub fn domain_mut(&mut self) -> &mut PropertyValues<Id, M> {
		&mut self.domain
	}

	pub fn required(&self) -> &FunctionalPropertyValue<bool, M> {
		&self.required
	}

	pub fn required_mut(&mut self) -> &mut FunctionalPropertyValue<bool, M> {
		&mut self.required
	}

	pub fn sub_property_of(&self) -> &PropertyValues<Id, M> {
		&self.sub_property_of
	}

	pub fn sub_property_of_mut(&mut self) -> &mut PropertyValues<Id, M> {
		&mut self.sub_property_of
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			domain: self.domain.iter(),
			range: self.range.iter(),
			required: self.required.iter(),
			sub_property_of: self.sub_property_of.iter(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		prop: RdfProperty,
		value: Meta<Object<M>, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			RdfProperty::Domain(p) => self
				.domain
				.insert(p, prop_cmp, rdf::from::expect_id(value)?),
			RdfProperty::Range(p) => self.range.insert(p, prop_cmp, rdf::from::expect_id(value)?),
			RdfProperty::SubPropertyOf(p) => {
				self.sub_property_of
					.insert(p, prop_cmp, rdf::from::expect_id(value)?)
			}
			RdfProperty::Required(p) => {
				self.required
					.insert(p, prop_cmp, rdf::from::expect_schema_boolean(value)?)
			}
		}

		Ok(())
	}

	pub(crate) fn build(
		&self,
		context: &crate::Context<M>,
		as_resource: &treeldr::node::Data<M>,
		meta: M,
	) -> Result<Meta<treeldr::prop::Definition<M>, M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let domain = self
			.domain
			.try_mapped(|_, Meta(domain_id, domain_meta)| {
				let domain_ref = context.require_type_id(*domain_id).map_err(|e| {
					e.at_node_property(
						as_resource.id,
						RdfProperty::Domain(None),
						domain_meta.clone(),
					)
				})?;
				Ok(Meta(domain_ref, domain_meta.clone()))
			})
			.map_err(|(Meta(e, _), _)| e)?;

		let range = self
			.range
			.try_mapped(|_, Meta(range_id, range_meta)| {
				let range_ref = context.require_type_id(*range_id).map_err(|e| {
					e.at_node_property(as_resource.id, RdfProperty::Range(None), range_meta.clone())
				})?;
				Ok(Meta(range_ref, range_meta.clone()))
			})
			.map_err(|(Meta(e, _), _)| e)?;

		let sub_property_of = self
			.sub_property_of
			.try_mapped(|_, Meta(prop_id, prop_meta)| {
				let prop_ref = context.require_property_id(*prop_id).map_err(|e| {
					e.at_node_property(
						as_resource.id,
						RdfProperty::SubPropertyOf(None),
						prop_meta.clone(),
					)
				})?;
				Ok(Meta(prop_ref, prop_meta.clone()))
			})
			.map_err(|(Meta(e, _), _)| e)?;

		let required = self.required.clone().try_unwrap().map_err(|e| {
			e.at_functional_node_property(as_resource.id, RdfProperty::Required(None))
		})?;

		let functional = match as_resource.type_metadata(context, Type::FunctionalProperty) {
			Some(meta) => Meta(true, meta.clone()),
			None => Meta(false, meta.clone()),
		};

		Ok(Meta(
			treeldr::prop::Definition::new(domain, range, sub_property_of, required, functional),
			meta,
		))
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<Property>) -> Id) {
		self.domain
			.map_ids_in(Some(RdfProperty::Domain(None).into()), &f);
		self.range
			.map_ids_in(Some(RdfProperty::Range(None).into()), f);
	}
}

#[derive(Debug)]
pub enum ClassBinding {
	Domain(Option<TId<UnknownProperty>>, Id),
	Range(Option<TId<UnknownProperty>>, Id),
	SubPropertyOf(Option<TId<UnknownProperty>>, Id),
	Required(Option<TId<UnknownProperty>>, bool),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> RdfProperty {
		match self {
			Self::Domain(p, _) => RdfProperty::Domain(*p),
			Self::Range(p, _) => RdfProperty::Range(*p),
			Self::SubPropertyOf(p, _) => RdfProperty::SubPropertyOf(*p),
			Self::Required(p, _) => RdfProperty::Required(*p),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Domain(_, v) => BindingValueRef::Id(*v),
			Self::Range(_, v) => BindingValueRef::Id(*v),
			Self::SubPropertyOf(_, v) => BindingValueRef::Id(*v),
			Self::Required(_, v) => BindingValueRef::Boolean(*v),
		}
	}
}

pub struct ClassBindings<'a, M> {
	domain: property_values::non_functional::Iter<'a, Id, M>,
	range: property_values::non_functional::Iter<'a, Id, M>,
	required: functional_property_value::Iter<'a, bool, M>,
	sub_property_of: property_values::non_functional::Iter<'a, Id, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.domain
			.next()
			.map(|m| m.into_cloned_class_binding(ClassBinding::Domain))
			.or_else(|| {
				self.range
					.next()
					.map(|m| m.into_cloned_class_binding(ClassBinding::Range))
			})
			.or_else(|| {
				self.required
					.next()
					.map(|m| m.into_cloned_class_binding(ClassBinding::Required))
			})
			.or_else(|| {
				self.sub_property_of
					.next()
					.map(|m| m.into_cloned_class_binding(ClassBinding::SubPropertyOf))
			})
	}
}
