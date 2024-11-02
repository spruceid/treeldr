use std::{cmp::Ordering, collections::{HashMap, HashSet}};

use disjoint_collections::DisjointVec;
use scc_trait::{Components, Scc};

use crate::{ty::{self, TypeUnificationError}, Registry, RegistryReservation, Type, TypeRef};

pub type TypeVariableRef = usize;

pub type TypePattern<R> = Type<R, TypeVariableRef>;

#[derive(Debug, Clone)]
pub enum TypeValue<R> {
	Undefined,
	Pattern(Type<R, TypeVariableRef>),
	Type(TypeRef),
}

impl<R> Default for TypeValue<R> {
	fn default() -> Self {
		Self::Undefined
	}
}

impl<R> TypeValue<R> {
	/// Unifies two type values.
	pub fn unify(
		self,
		registry: &Registry<R>,
		unifier: &mut Unifier,
		other: Self,
		// mut type_ref_variable: impl FnMut(&TypeRef) -> TypeVariableRef,
		// mut assert_eq: impl FnMut(TypeVariableRef, TypeVariableRef),
	) -> Result<Self, TypeUnificationError>
	where
		R: Clone + Ord,
	{
		match (self, other) {
			(Self::Undefined, b) => Ok(b),
			(a, Self::Undefined) => Ok(a),
			(Self::Pattern(a), Self::Pattern(b)) => a
				.unify_with(registry, unifier, &b)
				.map(Self::Pattern),
			(Self::Pattern(a), Self::Type(b_ref)) => {
				let b = registry.get_type(b_ref).unwrap();
				a.unify_with(registry, unifier, &b)
				.map(Self::Pattern)
			}
			(Self::Type(a_ref), Self::Pattern(b)) => {
				let a = registry.get_type(a_ref).unwrap();
				a.unify_with(registry, unifier, &b)
				.map(Self::Pattern)
			}
			(Self::Type(a), Self::Type(b)) => {
				if a == b {
					Ok(Self::Type(a.clone()))
				} else {
					Err(TypeUnificationError)
				}
			}
		}
	}
}

#[derive(Debug, thiserror::Error)]
#[error("undefined type")]
pub struct UndefinedType;

impl<R> From<Type<R, TypeVariableRef>> for TypeValue<R> {
	fn from(value: Type<R, TypeVariableRef>) -> Self {
		Self::Pattern(value)
	}
}

pub struct TypeVariables<R> {
	list: Vec<TypeValue<R>>,
	subtype_relation: SubtypeRelation,
	dependencies: ResolutionGroups,
	imported_types: HashMap<TypeRef, TypeVariableRef>,
}

impl<R> TypeVariables<R> {
	pub fn new() -> Self {
		Self {
			list: Vec::new(),
			subtype_relation: SubtypeRelation::default(),
			dependencies: ResolutionGroups::default(),
			imported_types: HashMap::new(),
		}
	}

	pub fn fresh(&mut self, value: impl Into<TypeValue<R>>) -> TypeVariableRef {
		todo!()
	}

	/// Import a type reference as a type variable.
	pub fn import_type_ref(&mut self, ty: &TypeRef) -> TypeVariableRef {
		let next_index = self.imported_types.len();
		*self.imported_types.entry(ty.clone()).or_insert(next_index)
	}

	/// Asserts that type `a` is a subtype of type `b`.
	pub fn assert_subtype_of(&mut self, a: TypeVariableRef, b: TypeVariableRef) {
		self.subtype_relation.insert(a, b);
	}

	/// Asserts that two type variables are equal.
	pub fn assert_eq(&mut self, a: TypeVariableRef, b: TypeVariableRef) {
		self.assert_subtype_of(a, b);
		self.assert_subtype_of(b, a);
	}

	/// Compute groups.
	/// 
	/// Most types can be resolved at the same time, with the exception of
	/// struct key types that must be resolved *before* the outer struct and
	/// *before* their associated value type.
	/// 
	/// For instance in the `struct { T: U }` type definition, `T` must be
	/// resolved *before* `struct { T: U }` and `U`.
	/// 
	/// This dependency is necessary to allow the unification of struct types,
	/// which requires knowing the keys to known what values to unify
	/// recursively.
	/// 
	/// # Relations
	/// 
	/// Type dependencies are specified using two relations:
	/// - The `=` relation: if `T = U` then `T` and `U` must be resolved
	///   together.
	/// - The `>` relation: if `T > U` then `T` must be resolved *before* `U`.
	/// 
	/// Here is the rules governing each relation:
	/// - if `T` is a non-field-key component of `U` then `T = U`.
	/// - if `T` is a subtype of `U` then `T = U`.
	/// - if `T: U` is an entry of the struct type `X`, then `T > X` and
	///   `T > U`.
	/// 
	/// The `=` relation is transitive, symmetric and reflexive.
	/// The `>` relation is transitive but *not* symmetric nor reflexive.
	/// 
	/// # Group
	/// 
	/// The `=` defines equivalence classes of type variables, called "groups",
	/// sets of variables such that for all class `X` and type variable `T`, `U`
	/// then
	/// ```text
	/// X ∋ T and X ∋ U ⇒ T = U
	/// ```
	/// We can extend the `>` relation to include groups such that
	/// ```text
	/// X ∋ T and Y ∋ U and T > U ⇒ X > Y
	/// ```
	fn compute_type_groups(
		&self,
		base_classes: &Components<usize>
	) -> Result<(ResolutionGroups, Vec<usize>), TypeResolutionError> {
		// Compute groups.
		let mut groups: DisjointVec<HashSet<TypeVariableRef>> = (0..self.list.len()).map(|x| {
			let mut set = HashSet::new();
			set.insert(x);
			set
		}).collect();

		for (x, value) in self.list.iter().enumerate() {
			for y in self.subtype_relation.supertypes_of(x) {
				groups.merge(x, y, |mut a, b| {
					a.extend(b);
					a
				});
			}

			if let TypeValue::Pattern(pattern) = value {
				pattern.visit_type_components(|&y, is_key| {
					if !is_key {
						groups.merge(x, y, |mut a, b| {
							a.extend(b);
							a
						});
					}
				});
			}
		}

		// Compute dependencies.
		let mut ordering: HashMap<usize, HashSet<usize>> = HashMap::new();
		for (c, _) in groups.classes() {
			ordering.insert(c, HashSet::new());
		}
		
		for (x, value) in self.list.iter().enumerate() {
			let x_class = groups.class_of(x).unwrap();
			if let TypeValue::Pattern(Type::Struct(s)) = value {
				for &y in s.fields.keys() {
					let y_class = groups.class_of(y).unwrap();
					ordering.get_mut(&y_class).unwrap().insert(x_class);
				}
			}
		}

		// Check that the dependencies are not symmetric nor reflexive and order
		// them by depth.
		let components = ordering.strongly_connected_components();
		let mut ordered_classes = Vec::with_capacity(components.len());
		for c in components.order_by_depth().into_iter() {
			match components.get_by_index(c).unwrap() {
				[class] if !components.is_cyclic(c) => {
					ordered_classes.push(*class);
				}
				_ => {
					return Err(TypeResolutionError::DependencyCycle)
				}
			}
		}

		Ok((
			ResolutionGroups {
				list: groups.map(|set| ResolutionGroup::new(base_classes, set)),
				ordering
			},
			ordered_classes
		))
	}

	/// Resolve the type variables.
	pub fn resolve(
		&self,
		registry: &mut Registry<R>,
	) -> Result<HashMap<TypeVariableRef, TypeRef>, TypeResolutionError>
	where
		R: Clone + Ord,
	{
		let (mut resolver, ordered_groups) = Resolver::new(self)?;

		// Resolve each group one by one, ordered by depth.
		let mut result: HashMap<TypeVariableRef, TypeRef> = HashMap::new();
		for g in ordered_groups {
			let group_result = resolver.resolve_group(registry, &result, g)?;
			result.extend(group_result);
		}

		// Resolve sub-typing relation.
		let super_types = self
			.subtype_relation
			.resolve(|x| result[&x]);

		// Check sub-type constraints.
		for (a, a_super_types) in &super_types {
			for &b in a_super_types {
				if !a
						.subtype_cmp(registry, b)
						.is_some_and(Ordering::is_le)
				{
					return Err(TypeResolutionError::Subtyping);
				}
			}
		}

		Ok(result)
	}
}

struct Resolver<'a, R> {
	/// Type variables.
	variables: &'a TypeVariables<R>,

	/// Resolution groups.
	groups: ResolutionGroups,

	/// Base equivalence classes.
	/// 
	/// The strongly connected components of the sub-typing relation graph.
	/// Types that are in the same base class must be unified.
	base_classes: Components<usize>,

	/// Equivalence classes.
	/// 
	/// When the resolution is done, this classification includes *at least* the
	/// base classification (`base_classes`).
	classes: DisjointVec<TypeValue<R>>,

	imported_types: Vec<TypeValue<R>>,
	imported_types_map: HashMap<TypeRef, TypeVariableRef>
}

impl<'a, R: Clone> Resolver<'a, R> {
	fn new(
		variables: &'a TypeVariables<R>
	) -> Result<(Self, Vec<usize>), TypeResolutionError> {
		let base_classes = variables
			.subtype_relation
			.strongly_connected_components();

		let (groups, ordered_groups) = variables
			.compute_type_groups(&base_classes)?;

		let classes: DisjointVec<TypeValue<R>> = variables
			.list
			.iter()
			.cloned()
			.collect();

		Ok((
			Self {
				variables,
				groups,
				base_classes,
				classes,
				imported_types: Vec::new(),
				imported_types_map: HashMap::new()
			},
			ordered_groups
		))
	}

	fn resolve_group(
		&mut self,
		registry: &mut Registry<R>,
		resolved: &HashMap<TypeVariableRef, TypeRef>,
		g: usize
	) -> Result<HashMap<TypeVariableRef, TypeRef>, TypeResolutionError>
	where
		R: Ord
	{
		let group = &self.groups.list.get(g).unwrap();

		// Prepare type unification. Types that are in the same base class must
		// be unified.
		let mut to_unify = Vec::new();

		for &c in &group.base_classes {
			let component = self.base_classes.get_by_index(c).unwrap();
			let (&first, rest) = component.split_first().unwrap();
			for &other in rest {
				to_unify.push((first, other));
			}
		}

		// Unify types.
		while let Some((a, b)) = to_unify.pop() {
			// let import_type_ref = |type_ref: &TypeRef| {
			// 	self.variables
			// 		.imported_types
			// 		.get(type_ref)
			// 		.copied()
			// 		.unwrap_or_else(|| {
			// 			*self.imported_types_map
			// 				.entry(type_ref.clone())
			// 				.or_insert_with(|| {
			// 					self.imported_types.push(TypeValue::Type(type_ref.clone()));
			// 					self.variables.list.len() + self.imported_types.len()
			// 				})
			// 		})
			// };

			// let assert_eq = |a, b| to_unify.push((a, b));

			let mut unifier = Unifier::new(resolved, &mut to_unify);
			self.classes.try_merge(a, b, |a, b| {
				// Unify `a` and `b`. This might trigger the unification of some
				// constituents of `a` and `b`. We know the constituents are
				// in the same resolution group.
				a.unify(registry, &mut unifier, b)
			})?;
		}

		// Resolve classes.
		let mut reservation = RegistryReservation::new(registry);
		let mut resolved_classes = HashMap::new();
		for &x in &group.variables {
			let (y, value) = self.classes.get_with_class(x).unwrap();
			if x == y {
				let target = match value {
					TypeValue::Undefined => return Err(TypeResolutionError::Definition(UndefinedType)),
					TypeValue::Type(r) => *r,
					TypeValue::Pattern(_) => reservation.reserve_type()
				};

				resolved_classes.insert(x, target);
			}
		}

		// Resolve variables from classes.
		let mut result = HashMap::new();
		for &x in &group.variables {
			let k = self.classes.class_of(x).unwrap();
			result.insert(x, resolved_classes[&k]);
		}

		// Realization.
		for (x, type_ref) in resolved_classes {
			if let TypeValue::Pattern(pattern) = self.classes.get(x).unwrap() {
				let type_ = pattern.realize(|x| result[x]);
				reservation.define_type(type_ref, type_);
			}
		}

		Ok(result)
	}
}

impl<R: Clone> TypePattern<R> {
	fn realize(&self, realize_var: impl Copy + Fn(&TypeVariableRef) -> TypeRef) -> Type<R> {
		match self {
			Type::Any => Type::Any,
			Type::Resource(r) => Type::Resource(r.clone()),
			Type::Literal(l) => Type::Literal(l.clone()),
			Type::List(l) => {
				Type::List(ty::List {
					prefix: l.prefix.iter().map(realize_var).collect(),
					rest: l.rest.as_ref().map(realize_var)
				})
			}
			_ => todo!()
		}
	}
}

struct Unifier<'a> {
	resolved: &'a HashMap<TypeVariableRef, TypeRef>,
	queue: &'a mut Vec<(TypeVariableRef, TypeVariableRef)>
}

impl<'a> Unifier<'a> {
	fn new(
		resolved: &'a HashMap<TypeVariableRef, TypeRef>,
		queue: &'a mut Vec<(TypeVariableRef, TypeVariableRef)>
	) -> Self {
		Self {
			resolved,
			queue
		}
	}
}

impl<'a> ty::TypeUnifier<TypeRef, TypeVariableRef> for Unifier<'a> {
	type Target = TypeVariableRef;

	fn left_key_type(&mut self, t: &TypeRef) -> TypeRef {
		*t
	}

	fn right_key_type(&mut self, u: &TypeVariableRef) -> TypeRef {
		self.resolved[u]
	}

	fn unify(&mut self, t: &TypeRef, u: &TypeVariableRef) -> Self::Target {
		todo!()
	}
}

impl<'a> ty::TypeUnifier<TypeVariableRef, TypeRef> for Unifier<'a> {
	type Target = TypeVariableRef;

	fn left_key_type(&mut self, t: &TypeVariableRef) -> TypeRef {
		self.resolved[t]
	}

	fn right_key_type(&mut self, u: &TypeRef) -> TypeRef {
		*u
	}

	fn unify(&mut self, t: &TypeVariableRef, u: &TypeRef) -> Self::Target {
		todo!()
	}
}

impl<'a> ty::TypeUnifier<TypeVariableRef, TypeVariableRef> for Unifier<'a> {
	type Target = TypeVariableRef;

	fn left_key_type(&mut self, t: &TypeVariableRef) -> TypeRef {
		self.resolved[t]
	}

	fn right_key_type(&mut self, u: &TypeVariableRef) -> TypeRef {
		self.resolved[u]
	}

	fn unify(&mut self, t: &TypeVariableRef, u: &TypeVariableRef) -> Self::Target {
		todo!()
	}
}

/// Subtype relation.
#[derive(Default)]
struct SubtypeRelation(HashMap<TypeVariableRef, HashSet<TypeVariableRef>>);

impl SubtypeRelation {
	/// Declares that `a` is a subtype of `b`.
	fn insert(&mut self, a: TypeVariableRef, b: TypeVariableRef) -> bool {
		self.0.entry(b).or_default().insert(a)
	}

	fn supertypes_of(&self, b: TypeVariableRef) -> impl '_ + Iterator<Item = TypeVariableRef> {
		self.0[&b].iter().copied()
	}

	fn resolve(
		&self,
		f: impl Copy + Fn(TypeVariableRef) -> TypeRef,
	) -> HashMap<TypeRef, HashSet<TypeRef>> {
		let mut result: HashMap<TypeRef, HashSet<TypeRef>> = HashMap::new();

		for (&key, value) in &self.0 {
			result
				.entry(f(key))
				.or_default()
				.extend(value.iter().copied().map(f));
		}

		result
	}
}

impl Scc for SubtypeRelation {
	type Vertex = TypeVariableRef;

	fn vertices(&self) -> impl '_ + IntoIterator<Item = Self::Vertex> {
		self.0.keys().copied()
	}

	fn successors(&self, v: Self::Vertex) -> impl '_ + IntoIterator<Item = Self::Vertex> {
		self.0[&v].iter().copied()
	}
}

#[derive(Default)]
struct ResolutionGroups {
	/// List of groups.
	list: DisjointVec<ResolutionGroup>,

	/// Group ordering.
	ordering: HashMap<usize, HashSet<usize>>
}

/// Resolution group.
struct ResolutionGroup {
	/// Type variables that are in the resolution group.
	variables: HashSet<TypeVariableRef>,

	/// Type variable base classes in the resolution group.
	base_classes: HashSet<usize>
}

impl ResolutionGroup {
	fn new(
		base_classes: &Components<usize>,
		variables: HashSet<TypeVariableRef>
	) -> Self {
		let group_classes = variables
			.iter()
			.map(|x| base_classes.vertex_component_index(x).unwrap())
			.collect();

		Self {
			variables,
			base_classes: group_classes
		}
	}
}

#[derive(Debug, thiserror::Error)]
enum TypeResolutionError {
	#[error("dependency cycle")]
	DependencyCycle,

	#[error("subtyping failed")]
	Subtyping,

	#[error(transparent)]
	Unification(#[from] TypeUnificationError),

	#[error(transparent)]
	Definition(#[from] UndefinedType)
}