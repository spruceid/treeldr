use core::fmt;
use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet},
	hash::Hash,
};

use disjoint_collections::DisjointVec;
use ll1::Span;
use scc_trait::{Components, Scc};

use crate::{
	ty::{self, TypeAllocator, TypeDesc, TypeUnificationError},
	Package, Registry, RegistryUnion, Type, TypeRef,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeVariableRef(pub usize);

impl fmt::Display for TypeVariableRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "T{}", self.0)
	}
}

pub type TypePattern<P, R> = Type<P, R, TypeVariableRef>;

#[derive(Debug, Clone)]
pub enum TypeValue<P, R> {
	Undefined,
	Pattern(Type<P, R, TypeVariableRef>),
	Type(ImportedTypeRef<P>),
}

impl<P, R> Default for TypeValue<P, R> {
	fn default() -> Self {
		Self::Undefined
	}
}

impl<P, R> TypeValue<P, R> {
	/// Unifies two type values.
	pub fn unify(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		unifier: &mut Unifier<P>,
		other: &Self,
		// mut type_ref_variable: impl FnMut(&TypeRef) -> TypeVariableRef,
		// mut assert_eq: impl FnMut(TypeVariableRef, TypeVariableRef),
	) -> Result<Self, TypeUnificationError>
	where
		P: Copy + Ord + Hash,
		R: Clone + Ord,
	{
		match (self, other) {
			(Self::Undefined, b) => Ok(b.clone()),
			(a, Self::Undefined) => Ok(a.clone()),
			(Self::Pattern(a), Self::Pattern(b)) => {
				a.unify_with(registry, unifier, b).map(Self::Pattern)
			}
			(Self::Pattern(a), Self::Type(b_ref)) => {
				let b = registry.get_type(b_ref.ty_ref).unwrap();
				a.unify_with(registry, &mut unifier.import(b_ref.import), b)
					.map(Self::Pattern)
			}
			(Self::Type(a_ref), Self::Pattern(b)) => {
				let a = registry.get_type(a_ref.ty_ref).unwrap();
				a.unify_with(registry, &mut unifier.import(a_ref.import), b)
					.map(Self::Pattern)
			}
			(Self::Type(a), Self::Type(b)) => {
				if a == b {
					Ok(Self::Type(*a))
				} else {
					Err(TypeUnificationError)
				}
			}
		}
	}
}

impl<P, R> From<Type<P, R, TypeVariableRef>> for TypeValue<P, R> {
	fn from(value: Type<P, R, TypeVariableRef>) -> Self {
		Self::Pattern(value)
	}
}

impl<P, R> From<ImportedTypeRef<P>> for TypeValue<P, R> {
	fn from(value: ImportedTypeRef<P>) -> Self {
		Self::Type(value)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImportedTypeRef<P> {
	pub ty_ref: TypeRef<P>,
	pub import: u32,
}

#[derive(Debug, thiserror::Error)]
#[error("undefined type")]
pub struct UndefinedType;

#[derive(Debug, Clone, Copy)]
pub struct TypeVariableMetadata {
	pub span: Option<Span>,
}

impl TypeVariableMetadata {
	pub fn new(span: Option<Span>) -> Self {
		Self { span }
	}

	pub fn unify(self, other: Self) -> Self {
		Self {
			span: self.span.or(other.span),
		}
	}
}

pub struct DisjointTypeVariables<T>(DisjointVec<T>);

impl<T> DisjointTypeVariables<T> {
	pub fn new<P, R>(
		variables: &TypeVariables<P, R>,
		mut f: impl FnMut(TypeVariableRef, &TypeValue<P, R>) -> T,
	) -> Self {
		Self(variables.values().map(|(x, value)| f(x, value)).collect())
	}

	pub fn get(&self, i: usize) -> Option<&T> {
		self.0.get(i)
	}

	pub fn merge(&mut self, x: TypeVariableRef, y: TypeVariableRef, f: impl FnOnce(T, T) -> T) {
		self.0.merge(x.0, y.0, f);
	}

	pub fn try_merge<E>(
		&mut self,
		x: TypeVariableRef,
		y: TypeVariableRef,
		f: impl FnOnce(T, T) -> Result<T, E>,
	) -> Result<(), E> {
		self.0.try_merge(x.0, y.0, f)?;
		Ok(())
	}

	pub fn classes(&self) -> impl Iterator<Item = (usize, &T)> {
		self.0.classes()
	}

	pub fn class_of(&self, x: TypeVariableRef) -> Option<usize> {
		self.0.class_of(x.0)
	}

	pub fn get_with_class(&self, x: TypeVariableRef) -> Option<(usize, &T)> {
		self.0.get_with_class(x.0)
	}

	pub fn map<U>(self, f: impl FnMut(T) -> U) -> DisjointTypeVariables<U> {
		DisjointTypeVariables(self.0.map(f))
	}
}

pub struct TypeVariables<P, R> {
	list: Vec<TypeValue<P, R>>,
	subtype_relation: SubtypeRelation,
	import_count: u32,
	imported_types: HashMap<ImportedTypeRef<P>, TypeVariableRef>,
	metadata: HashMap<TypeVariableRef, TypeVariableMetadata>,
}

impl<P, R> Default for TypeVariables<P, R> {
	fn default() -> Self {
		Self {
			list: Vec::new(),
			subtype_relation: SubtypeRelation::default(),
			import_count: 0,
			imported_types: HashMap::new(),
			metadata: HashMap::new(),
		}
	}
}

impl<P, R> TypeVariables<P, R> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn iter(&self) -> impl '_ + Iterator<Item = TypeVariableRef> {
		(0..self.list.len()).map(TypeVariableRef)
	}

	pub fn values(&self) -> impl Iterator<Item = (TypeVariableRef, &TypeValue<P, R>)> {
		self.list
			.iter()
			.enumerate()
			.map(|(x, value)| (TypeVariableRef(x), value))
	}

	pub fn fresh(
		&mut self,
		value: impl Into<TypeValue<P, R>>,
		span: Option<Span>,
	) -> TypeVariableRef {
		let x = TypeVariableRef(self.list.len());
		self.list.push(value.into());
		self.subtype_relation.declare(x);
		self.metadata.insert(x, TypeVariableMetadata::new(span));
		x
	}

	pub fn metadata_of(&self, x: TypeVariableRef) -> Option<TypeVariableMetadata> {
		self.metadata.get(&x).copied()
	}

	/// Import a type reference as a type variable.
	pub fn import_type_ref(
		&mut self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		ty_ref: TypeRef<P>,
	) -> TypeVariableRef
	where
		P: Copy + Ord + Hash,
	{
		let import = self.import_count;
		self.import_count += 1;

		let mut stack = vec![ty_ref];
		let mut visited = HashSet::new();

		while let Some(ty_ref) = stack.pop() {
			if visited.insert(ty_ref) {
				let imported = ImportedTypeRef { ty_ref, import };

				let x = self.fresh(imported, None);
				self.imported_types.insert(imported, x);

				let ty = registry.get_type(ty_ref).unwrap();
				ty.visit_type_components(|&c_ref, _| stack.push(c_ref));
			}
		}

		self.imported_types[&ImportedTypeRef { ty_ref, import }]
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

	/// Compute resolution groups.
	///
	/// Most types can be resolved at the same time, with the exception of
	/// struct key types that must be resolved *before* the outer struct and
	/// *before* their associated value type.
	///
	/// For instance in the `struct { T: U }` type definition, `T` must be
	/// resolved *before* `struct { T: U }` and `U`.
	///
	/// This dependency is necessary to allow the unification of struct types,
	/// which requires knowing the keys in order to known what values to unify
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
	fn compute_resolution_groups(
		&self,
		base_classes: &BaseClasses,
	) -> Result<(DisjointTypeVariables<ResolutionGroup>, Vec<usize>), TypeResolutionError<P>> {
		// Compute groups.
		let mut groups = DisjointTypeVariables::new(self, |x, _| {
			let mut set = HashSet::new();
			set.insert(x);
			set
		});

		for (x, value) in self.values() {
			for y in self.subtype_relation.subtypes_of(x) {
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

		for (x, value) in self.values() {
			let x_class = groups.class_of(x).unwrap();
			if let TypeValue::Pattern(Type {
				desc: TypeDesc::Map(s),
				..
			}) = value
			{
				for &y in s.fields.keys() {
					let y_class = groups.class_of(y).unwrap();
					ordering.get_mut(&y_class).unwrap().insert(x_class);
				}
			}
		}

		// Check that the dependencies are not symmetric nor reflexive and order
		// them by depth.
		let components = ordering.strongly_connected_components();
		let mut ordered_groups = Vec::with_capacity(components.len());
		for c in components.order_by_depth().into_iter() {
			match components.get_by_index(c).unwrap() {
				[g] if !components.is_cyclic(c) => {
					ordered_groups.push(*g);
				}
				_ => return Err(TypeResolutionError::DependencyCycle),
			}
		}

		Ok((
			groups.map(|set| ResolutionGroup::new(base_classes, set)),
			ordered_groups,
		))
	}

	/// Resolve the type variables.
	pub fn resolve(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		package: &mut Package<P, R>,
	) -> Result<HashMap<TypeVariableRef, TypeRef<P>>, TypeResolutionError<P>>
	where
		P: Copy + Ord + Hash,
		R: Clone + Ord,
	{
		let (mut resolver, ordered_groups) = Resolver::new(self)?;

		// Resolve each group one by one, ordered by depth.
		let mut result: HashMap<TypeVariableRef, TypeRef<P>> = HashMap::new();
		for g in ordered_groups {
			let group_result = resolver.resolve_group(registry, package, &result, g)?;
			result.extend(group_result);
		}

		// Resolve sub-typing relation.
		let subtypes_types = self.subtype_relation.resolve(|x| result[&x]);

		let combined_registry = RegistryUnion(registry, package);

		// Check sub-type constraints.
		for (&a, a_subtypes) in &subtypes_types {
			for &b in a_subtypes {
				if !b
					.subtype_cmp(&combined_registry, a)
					.is_some_and(Ordering::is_le)
				{
					return Err(TypeResolutionError::Subtyping {
						sub_type: b,
						super_type: a,
					});
				}
			}
		}

		Ok(result)
	}
}

struct Resolver<'a, P, R> {
	/// Type variables.
	variables: &'a TypeVariables<P, R>,

	/// Resolution groups.
	groups: DisjointTypeVariables<ResolutionGroup>,

	/// Base equivalence classes.
	///
	/// The strongly connected components of the sub-typing relation graph.
	/// Types that are in the same base class must be unified.
	base_classes: BaseClasses,

	/// Equivalence classes.
	///
	/// When the resolution is done, this classification includes *at least* the
	/// base classification (`base_classes`).
	classes: Classes<P, R>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BaseClassRef(usize);

impl fmt::Display for BaseClassRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "B{}", self.0)
	}
}

/// Base equivalence classes.
///
/// The strongly connected components of the sub-typing relation graph.
/// Types that are in the same base class must be unified.
pub struct BaseClasses {
	components: Components<TypeVariableRef>,
	depths: Vec<usize>,
}

impl BaseClasses {
	fn new(components: Components<TypeVariableRef>) -> Self {
		let depths = components.depths();

		Self { components, depths }
	}

	fn get(&self, c: BaseClassRef) -> Option<&[TypeVariableRef]> {
		self.components.get_by_index(c.0)
	}

	fn of(&self, x: TypeVariableRef) -> Option<BaseClassRef> {
		self.components.vertex_component_index(&x).map(BaseClassRef)
	}

	fn depth(&self, c: BaseClassRef) -> Option<usize> {
		self.depths.get(c.0).copied()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ClassRef(usize);

impl fmt::Display for ClassRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "C{}", self.0)
	}
}

/// Equivalence classes.
struct Classes<P, R>(DisjointTypeVariables<Class<P, R>>);

impl<P, R> Classes<P, R> {
	fn new(variables: &TypeVariables<P, R>) -> Self
	where
		P: Clone,
		R: Clone,
	{
		Self(DisjointTypeVariables::new(variables, |x, value| {
			Class::new(x, variables.metadata_of(x).unwrap(), value.clone())
		}))
	}

	fn get(&self, k: ClassRef) -> Option<&Class<P, R>> {
		self.0.get(k.0)
	}

	fn of(&self, x: TypeVariableRef) -> Option<ClassRef> {
		self.0.class_of(x).map(ClassRef)
	}

	fn of_with_value(&self, x: TypeVariableRef) -> Option<(ClassRef, &Class<P, R>)> {
		self.0
			.get_with_class(x)
			.map(|(i, value)| (ClassRef(i), value))
	}

	fn try_merge<E>(
		&mut self,
		x: TypeVariableRef,
		y: TypeVariableRef,
		f: impl FnOnce(Class<P, R>, Class<P, R>) -> Result<Class<P, R>, E>,
	) -> Result<(), E> {
		self.0.try_merge(x, y, f)
	}
}

struct Class<P, R> {
	variables: Vec<TypeVariableRef>,
	metadata: TypeVariableMetadata,
	value: TypeValue<P, R>,
}

impl<P, R> Class<P, R> {
	fn new(x: TypeVariableRef, metadata: TypeVariableMetadata, value: TypeValue<P, R>) -> Self {
		Self {
			variables: vec![x],
			metadata,
			value,
		}
	}

	fn unify(
		mut self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		unifier: &mut Unifier<P>,
		mut other: Self,
	) -> Result<Self, TypeResolutionError<P>>
	where
		P: Copy + Ord + Hash,
		R: Clone + Ord,
	{
		self.variables.append(&mut other.variables);
		Ok(Self {
			variables: self.variables,
			metadata: self.metadata.unify(other.metadata),
			value: self.value.unify(registry, unifier, &other.value).map_err(
				|TypeUnificationError| {
					TypeResolutionError::Unification(self.metadata, other.metadata)
				},
			)?,
		})
	}
}

impl<'a, P: Copy, R: Clone> Resolver<'a, P, R> {
	fn new(
		variables: &'a TypeVariables<P, R>,
	) -> Result<(Self, Vec<usize>), TypeResolutionError<P>> {
		let base_classes = variables.subtype_relation.base_classes();

		let (groups, ordered_groups) = variables.compute_resolution_groups(&base_classes)?;

		let classes = Classes::new(variables);

		Ok((
			Self {
				variables,
				groups,
				base_classes,
				classes,
			},
			ordered_groups,
		))
	}

	fn resolve_group(
		&mut self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		package: &mut Package<P, R>,
		resolved: &HashMap<TypeVariableRef, TypeRef<P>>,
		g: usize,
	) -> Result<HashMap<TypeVariableRef, TypeRef<P>>, TypeResolutionError<P>>
	where
		P: Ord + Hash,
		R: Ord,
	{
		eprintln!("group {g}");
		let group = &self.groups.get(g).unwrap();

		// Prepare type unification.
		let mut to_unify = Vec::new();

		// Types that are in the same base class must be unified.
		for &c in &group.base_classes {
			let component = self.base_classes.get(c).unwrap();
			let (&first, rest) = component.split_first().unwrap();
			for &other in rest {
				to_unify.push((first, other, (first, other)));
			}
		}

		// Unify types.
		while let Some((a, b, cause)) = to_unify.pop() {
			let mut unifier = Unifier::new(
				&self.variables.imported_types,
				resolved,
				&mut to_unify,
				cause,
			);

			self.classes.try_merge(a, b, |a, b| {
				// Unify `a` and `b`. This might trigger the unification of some
				// constituents of `a` and `b`. We know the constituents are
				// in the same resolution group.
				a.unify(registry, &mut unifier, b)
			})?;
		}

		// Order the variables by base class depth, so that we visit the super
		// types before their sub types while resolving the classes below.
		let variables = group
			.base_classes
			.iter()
			.flat_map(|&c| self.base_classes.get(c).unwrap());

		for &b in &group.base_classes {
			eprint!("{b} = {{");
			let base_class = self.base_classes.get(b).unwrap();
			for x in base_class {
				eprint!("{x},");
			}
			eprintln!("}}");
		}

		// Resolve classes.
		let mut reservation = TypeAllocator::new(package);
		let mut resolved_classes: HashMap<ClassRef, TypeRef<P>> = HashMap::new();

		for &x in variables {
			let (k, class) = self.classes.of_with_value(x).unwrap();

			if resolved_classes.contains_key(&k) {
				continue;
			}

			eprint!("resolve {k} = {{");
			for &x in &class.variables {
				eprint!("{x},");
			}
			eprintln!("}}");

			let target = match &class.value {
				TypeValue::Undefined => {
					// The type is undefined, but we can try to fallback to
					// a super-type if it is un-ambiguous.
					let mut candidates: Vec<_> = class
						.variables
						.iter()
						.copied()
						.flat_map(|y| {
							self.variables
								.subtype_relation
								.supertypes_of(y)
								.map(|z| {
									eprintln!("parent type {z}");
									self.classes.of(z).unwrap()
								})
								.filter(|&l| k != l)
								.map(|l| {
									eprintln!("parent class {l}");
									*resolved_classes.get(&l).unwrap()
								})
						})
						.collect();

					// let mut candidates: Vec<_> = self.variables.subtype_relation.supertypes_of(x).filter_map(|y| {
					// 	self.classes.of(y)
					// }).filter(|&k2| k2 != k).collect();
					candidates.sort_unstable();
					candidates.dedup();

					match candidates.as_slice() {
						[z] => *z,
						// Ambiguous or empty (type annotation needed).
						_ => return Err(TypeResolutionError::Definition(class.metadata.span)),
					}

					// return Err(TypeResolutionError::Definition(class.metadata.span))
				}
				TypeValue::Type(r) => r.ty_ref,
				TypeValue::Pattern(_) => reservation.reserve_type(),
			};

			resolved_classes.insert(k, target);
		}

		// Resolve variables from classes.
		let mut result = HashMap::new();
		for &x in &group.variables {
			let k = self.classes.of(x).unwrap();
			result.insert(x, resolved_classes[&k]);
		}

		// Realization.
		for (k, type_ref) in resolved_classes {
			if let TypeValue::Pattern(pattern) = &self.classes.get(k).unwrap().value {
				let type_ =
					pattern.realize(|x| resolved.get(x).copied().unwrap_or_else(|| result[x]));
				reservation.define_type(type_ref, type_);
			}
		}

		Ok(result)
	}
}

impl<P: Ord, R: Clone> TypePattern<P, R> {
	fn realize(&self, realize_var: impl Copy + Fn(&TypeVariableRef) -> TypeRef<P>) -> Type<P, R> {
		let desc = match &self.desc {
			TypeDesc::Any => TypeDesc::Any,
			TypeDesc::Resource(r) => TypeDesc::Resource(r.clone()),
			TypeDesc::Literal(l) => TypeDesc::Literal(l.clone()),
			TypeDesc::List(l) => TypeDesc::List(ty::List::new(
				l.prefix.iter().map(realize_var).collect(),
				l.rest.as_ref().map(realize_var),
			)),
			TypeDesc::Map(ty) => TypeDesc::Map(ty::Map {
				fields: ty
					.fields
					.iter()
					.map(|(key, field)| {
						(
							realize_var(key),
							ty::map::Field::new(
								field.attributes.clone(),
								realize_var(&field.r#type),
								field.required,
							),
						)
					})
					.collect(),
			}),
			TypeDesc::Enum(ty) => TypeDesc::Enum(ty::Enum {
				variants: ty
					.variants
					.iter()
					.map(|(name, v)| {
						let variant =
							ty::r#enum::Variant::new(v.attributes.clone(), realize_var(&v.r#type));
						(name.clone(), variant)
					})
					.collect(),
			}),
		};

		Type::new(self.attributes.clone(), desc)
	}
}

pub type UnificationCause = (TypeVariableRef, TypeVariableRef);

pub struct Unifier<'a, P> {
	imported_types: &'a HashMap<ImportedTypeRef<P>, TypeVariableRef>,
	resolved: &'a HashMap<TypeVariableRef, TypeRef<P>>,
	queue: &'a mut Vec<(TypeVariableRef, TypeVariableRef, UnificationCause)>,
	cause: UnificationCause,
}

impl<'a, P> Unifier<'a, P> {
	fn new(
		imported_types: &'a HashMap<ImportedTypeRef<P>, TypeVariableRef>,
		resolved: &'a HashMap<TypeVariableRef, TypeRef<P>>,
		queue: &'a mut Vec<(TypeVariableRef, TypeVariableRef, UnificationCause)>,
		cause: UnificationCause,
	) -> Self {
		Self {
			imported_types,
			resolved,
			queue,
			cause,
		}
	}

	fn import(&mut self, import: u32) -> ImportUnifier<'a, '_, P> {
		ImportUnifier {
			unifier: self,
			import,
		}
	}
}

impl<P: Copy + Eq + Hash> ty::TypeUnifier<P, TypeVariableRef, TypeVariableRef> for Unifier<'_, P> {
	type Target = TypeVariableRef;

	fn left_key_type(&mut self, t: &TypeVariableRef) -> TypeRef<P> {
		self.resolved[t]
	}

	fn right_key_type(&mut self, u: &TypeVariableRef) -> TypeRef<P> {
		self.resolved[u]
	}

	fn unify(&mut self, t: &TypeVariableRef, u: &TypeVariableRef) -> Self::Target {
		self.queue.push((*t, *u, self.cause));
		*t
	}
}

struct ImportUnifier<'a, 'u, P> {
	unifier: &'u mut Unifier<'a, P>,
	import: u32,
}

impl<P: Copy + Eq + Hash> ty::TypeUnifier<P, TypeRef<P>, TypeVariableRef>
	for ImportUnifier<'_, '_, P>
{
	type Target = TypeVariableRef;

	fn left_key_type(&mut self, t: &TypeRef<P>) -> TypeRef<P> {
		*t
	}

	fn right_key_type(&mut self, u: &TypeVariableRef) -> TypeRef<P> {
		self.unifier.resolved[u]
	}

	fn unify(&mut self, t: &TypeRef<P>, u: &TypeVariableRef) -> Self::Target {
		let t = self.unifier.imported_types[&ImportedTypeRef {
			ty_ref: *t,
			import: self.import,
		}];
		self.unifier.queue.push((t, *u, self.unifier.cause));
		t
	}
}

impl<P: Copy + Eq + Hash> ty::TypeUnifier<P, TypeVariableRef, TypeRef<P>>
	for ImportUnifier<'_, '_, P>
{
	type Target = TypeVariableRef;

	fn left_key_type(&mut self, t: &TypeVariableRef) -> TypeRef<P> {
		self.unifier.resolved[t]
	}

	fn right_key_type(&mut self, u: &TypeRef<P>) -> TypeRef<P> {
		*u
	}

	fn unify(&mut self, t: &TypeVariableRef, u: &TypeRef<P>) -> Self::Target {
		let u = self.unifier.imported_types[&ImportedTypeRef {
			ty_ref: *u,
			import: self.import,
		}];
		self.unifier.queue.push((*t, u, self.unifier.cause));
		*t
	}
}

/// Subtype relation.
#[derive(Default)]
struct SubtypeRelation {
	super_to_sub: HashMap<TypeVariableRef, HashSet<TypeVariableRef>>,
	sub_to_super: HashMap<TypeVariableRef, HashSet<TypeVariableRef>>,
}

impl SubtypeRelation {
	fn base_classes(&self) -> BaseClasses {
		BaseClasses::new(self.strongly_connected_components())
	}

	fn declare(&mut self, x: TypeVariableRef) {
		eprintln!("{x}");
		self.super_to_sub.entry(x).or_default();
		self.sub_to_super.entry(x).or_default();
	}

	/// Declares that `a` is a subtype of `b`.
	fn insert(&mut self, a: TypeVariableRef, b: TypeVariableRef) -> bool {
		eprintln!("{a} < {b}");
		self.super_to_sub.get_mut(&b).unwrap().insert(a);
		self.sub_to_super.get_mut(&a).unwrap().insert(b)
	}

	fn subtypes_of(&self, b: TypeVariableRef) -> impl '_ + Iterator<Item = TypeVariableRef> {
		self.super_to_sub[&b].iter().copied()
	}

	fn supertypes_of(&self, a: TypeVariableRef) -> impl '_ + Iterator<Item = TypeVariableRef> {
		self.sub_to_super[&a].iter().copied()
	}

	fn resolve<P: Eq + Hash>(
		&self,
		f: impl Copy + Fn(TypeVariableRef) -> TypeRef<P>,
	) -> HashMap<TypeRef<P>, HashSet<TypeRef<P>>> {
		let mut result: HashMap<TypeRef<P>, HashSet<TypeRef<P>>> = HashMap::new();

		for (&key, value) in &self.super_to_sub {
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
		self.super_to_sub.keys().copied()
	}

	fn successors(&self, v: Self::Vertex) -> impl '_ + IntoIterator<Item = Self::Vertex> {
		self.super_to_sub[&v].iter().copied()
	}
}

/// Resolution group.
struct ResolutionGroup {
	/// Type variables that are in the resolution group.
	variables: HashSet<TypeVariableRef>,

	/// Type variable base classes in the resolution group.
	///
	/// Ordered by sub-type (super-types first).
	base_classes: Vec<BaseClassRef>,
}

impl ResolutionGroup {
	fn new(base_classes: &BaseClasses, variables: HashSet<TypeVariableRef>) -> Self {
		let mut group_classes: Vec<_> = variables
			.iter()
			.copied()
			.map(|x| base_classes.of(x).unwrap())
			.collect();

		group_classes.sort_by_key(|&c| base_classes.depth(c).unwrap());
		group_classes.dedup();

		Self {
			variables,
			base_classes: group_classes,
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum TypeResolutionError<P> {
	#[error("dependency cycle")]
	DependencyCycle,

	#[error("subtyping failed")]
	Subtyping {
		sub_type: TypeRef<P>,
		super_type: TypeRef<P>,
	},

	#[error("type unification failed")]
	Unification(TypeVariableMetadata, TypeVariableMetadata),

	#[error("type annotation needed")]
	Definition(Option<Span>),
}
