use std::{cell::RefCell, hash::Hash, marker::PhantomData};

use crate::{
	build,
	function::FunctionRef,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, TryUnifyInterpretation, VisitResources},
	syntax,
	ty::TypeDefinition,
	Function, Module, ModuleRef, PackageId, Registry, Type, TypeRef,
};
use educe::Educe;
use rdf_types::{interpretation::ReverseInterpretation, InterpretationMut, LocalTerm};
use serde::{ser::SerializeSeq, Deserialize, Serialize};
use serde_seeded::{de::Seed, ser::Seeded, unseeded, DeserializeSeeded, SerializeSeeded};

/// Package.
///
/// A package is a collection of types, functions and modules.
#[derive(Debug, SerializeSeeded, DeserializeSeeded)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord), bounds(P: Ord))
)]
pub struct Package<P, R = rdf_types::LocalTerm> {
	#[seeded(with(unseeded))]
	pub id: P,

	/// Types.
	pub types: Collection<TypeDefinition<P, R>>,

	/// Functions.
	pub functions: Collection<Function<P, R>>,

	/// Modules.
	pub modules: NonEmptyCollection<Module<P, R>>,
}

impl<P, R> Package<P, R> {
	pub fn new(id: P) -> Self {
		Self::new_with_root(id, Module::new())
	}

	pub fn new_with_root(id: P, root: Module<P, R>) -> Self {
		Self {
			id,
			types: Collection::default(),
			functions: Collection::default(),
			modules: NonEmptyCollection::new(root),
		}
	}

	pub fn root(&self) -> &Module<P, R> {
		self.modules.first()
	}

	pub fn root_mut(&mut self) -> &mut Module<P, R> {
		self.modules.first_mut()
	}

	pub fn set_root(&mut self, value: Module<P, R>) -> Module<P, R> {
		std::mem::replace(self.root_mut(), value)
	}

	pub fn with_interpretation<'a, C>(&self, interpretation: &'a C) -> Seeded<'a, C, &Self> {
		Seeded::new(interpretation, self)
	}
}

impl<P, R> VisitResources<R> for Package<P, R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.types.visit_resources_ref_mut(f);
		self.functions.visit_resources_ref_mut(f);
		self.modules.visit_resources_ref_mut(f);
	}
}

impl<P: Copy, R> Package<P, R> {
	pub fn root_ref(&self) -> ModuleRef<P> {
		ModuleRef::new(self.id, 0)
	}

	pub fn insert_type(&mut self, value: impl Into<Type<P, R>>) -> TypeRef<P> {
		TypeRef::new(self.id, self.types.insert(value.into()))
	}

	pub fn insert_function(&mut self, value: Function<P, R>) -> FunctionRef<P> {
		FunctionRef::new(self.id, self.functions.insert(value))
	}

	pub fn insert_module(&mut self, value: Module<P, R>) -> ModuleRef<P> {
		ModuleRef::new(self.id, self.modules.insert(value))
	}
}

impl<P: PackageId + Ord + Hash, R: Clone + Ord + Hash> Package<P, R> {
	/// Compiles a package.
	pub fn compile_with(
		rdf: &mut impl InterpretationMut<Resource = R>,
		registry: &(impl ?Sized + Registry<PackageId = P, Resource = R>),
		package_id: P,
		text: &str,
	) -> Result<Self, CompileError<P>> {
		use crate::build::{self, BuildContext, BuildingPackage};
		use crate::syntax::{self, Parse};

		let ast = syntax::ast::Module::parse_str(text)?;

		let mut namespace = BuildingPackage::new(package_id);
		let mut context = BuildContext::new(rdf, registry);

		let scope = build::Scope::new();

		ast.declare(&mut context, &mut namespace, &scope, None)?;

		namespace
			.build(&mut context)?
			.resolve(context)
			.map_err(Into::into)
	}
}

impl<P: PackageId + Ord + Hash> Package<P> {
	/// Compiles a package.
	pub fn compile(
		registry: &(impl ?Sized + Registry<PackageId = P, Resource = LocalTerm>),
		package_id: P,
		text: &str,
	) -> Result<Self, CompileError<P>> {
		Self::compile_with(&mut (), registry, package_id, text)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError<P> {
	#[error(transparent)]
	Parse(#[from] syntax::ParseError),

	#[error(transparent)]
	Build(#[from] build::Error<P>),

	#[error(transparent)]
	Typing(#[from] build::TypeResolutionError<P>),
}

#[derive(Debug, Educe)]
#[educe(Default)]
pub struct Collection<T>(Vec<T>);

impl<T> Collection<T> {
	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn iter(&self) -> std::slice::Iter<T> {
		self.0.iter()
	}

	pub fn get(&self, i: u32) -> Option<&T> {
		self.0.get(i as usize)
	}

	pub fn get_mut(&mut self, i: u32) -> Option<&mut T> {
		self.0.get_mut(i as usize)
	}

	/// Inserts a new item into the collection and returns its index.
	///
	/// The returned index is guaranteed to be equal to `self.len()`.
	pub fn insert(&mut self, value: impl Into<T>) -> u32 {
		let i = self.0.len() as u32;
		self.0.push(value.into());
		i
	}

	pub fn replace(&mut self, i: u32, value: T) -> Result<T, T> {
		match self.0.get_mut(i as usize) {
			Some(t) => Ok(std::mem::replace(t, value)),
			None => Err(value),
		}
	}
}

impl<'a, T> IntoIterator for &'a Collection<T> {
	type IntoIter = std::slice::Iter<'a, T>;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<R, T: VisitResources<R>> VisitResources<R> for Collection<T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		for item in self {
			item.visit_resources_ref_mut(f);
		}
	}
}

impl<T: SerializeSeeded<Q>, Q> SerializeSeeded<Q> for Collection<T> {
	fn serialize_seeded<S>(&self, seed: &Q, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.serialize_seeded(seed, serializer)
	}
}

impl<'de, T: DeserializeSeeded<'de, Q>, Q> DeserializeSeeded<'de, Q> for Collection<T> {
	fn deserialize_seeded<D>(seed: &Q, deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Vec::deserialize_seeded(seed, deserializer).map(Self)
	}
}

#[derive(Debug)]
pub struct NonEmptyCollection<T>(Vec<T>);

impl<T> NonEmptyCollection<T> {
	pub fn new(first: T) -> Self {
		Self(vec![first])
	}

	pub fn iter(&self) -> std::slice::Iter<T> {
		self.0.iter()
	}

	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn first(&self) -> &T {
		self.0.first().unwrap()
	}

	pub fn first_mut(&mut self) -> &mut T {
		self.0.first_mut().unwrap()
	}

	pub fn get(&self, i: u32) -> Option<&T> {
		self.0.get(i as usize)
	}

	pub fn get_mut(&mut self, i: u32) -> Option<&mut T> {
		self.0.get_mut(i as usize)
	}

	/// Inserts a new item into the collection and returns its index.
	///
	/// The returned index is guaranteed to be equal to `self.len()`.
	pub fn insert(&mut self, value: impl Into<T>) -> u32 {
		let i = self.0.len() as u32;
		self.0.push(value.into());
		i
	}

	pub fn replace(&mut self, i: u32, value: T) -> Result<T, T> {
		match self.0.get_mut(i as usize) {
			Some(t) => Ok(std::mem::replace(t, value)),
			None => Err(value),
		}
	}
}

impl<'a, T> IntoIterator for &'a NonEmptyCollection<T> {
	type IntoIter = std::slice::Iter<'a, T>;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<R, T: VisitResources<R>> VisitResources<R> for NonEmptyCollection<T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		for item in self {
			item.visit_resources_ref_mut(f);
		}
	}
}

impl<T: SerializeSeeded<Q>, Q> SerializeSeeded<Q> for NonEmptyCollection<T> {
	fn serialize_seeded<S>(&self, index: &Q, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.serialize_seeded(index, serializer)
	}
}

impl<'de, T: DeserializeSeeded<'de, Q>, Q> DeserializeSeeded<'de, Q> for NonEmptyCollection<T> {
	fn deserialize_seeded<D>(index: &Q, deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let inner = Vec::deserialize_seeded(index, deserializer)?;

		if inner.is_empty() {
			return Err(serde::de::Error::custom("empty collection"));
		}

		Ok(Self(inner))
	}
}

impl<P: PackageId + PartialEq, R> Registry for Package<P, R> {
	type PackageId = P;
	type Resource = R;

	fn len(&self) -> usize {
		1
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		if id == self.id {
			Some(self)
		} else {
			None
		}
	}

	fn prelude_package_id(&self) -> Option<Self::PackageId> {
		Some(self.id)
	}
}

impl<C, P, R> SerializeSeeded<C> for Package<P, R>
where
	C: ReverseInterpretation<Resource = R>,
	P: Serialize,
	R: Clone + Eq + Hash,
{
	fn serialize_seeded<S>(&self, rdf: &C, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut index = ResourceForwardIndex::new();
		self.visit_resources(|r| {
			index.insert(r.clone());
		});

		let mut s = serializer.serialize_seq(Some(2))?;

		s.serialize_element(&Seeded::new(rdf, &index))?;
		s.serialize_element(&Seeded::new(&index, self))?;

		s.end()
	}
}

impl<'de, C, P, R> DeserializeSeeded<'de, RefCell<C>> for Package<P, R>
where
	C: InterpretationMut<Resource = R> + TryUnifyInterpretation,
	P: Ord + Deserialize<'de>,
	R: Clone + Ord,
{
	fn deserialize_seeded<D>(rdf: &RefCell<C>, deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor<'seed, C, P, R>(&'seed RefCell<C>, PhantomData<(P, R)>);

		impl<'de, C, P, R> serde::de::Visitor<'de> for Visitor<'_, C, P, R>
		where
			C: InterpretationMut<Resource = R> + TryUnifyInterpretation,
			P: Ord + Deserialize<'de>,
			R: Clone + Ord,
		{
			type Value = Package<P, R>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a pair")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let index: ResourceBackwardIndex<R> = seq
					.next_element_seed(Seed::new(self.0))?
					.ok_or_else(|| ::serde::de::Error::custom("missing resource index"))?;
				seq.next_element_seed(Seed::new(&index))?
					.ok_or_else(|| ::serde::de::Error::custom("missing package"))
			}
		}

		deserializer.deserialize_seq(Visitor(rdf, PhantomData))
	}
}
