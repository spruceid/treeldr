use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use rdf_types::Term;
use serde::{Deserialize, Serialize};

use crate::{
	function::FunctionRef,
	module::{ItemPath, ItemPathSegment, ItemRef},
	package::CompileError,
	rdf::{Rdf, RdfContextMut},
	ty::TypeDefinition,
	Function, Module, ModuleRef, Package, Type, TypeRef,
};

pub trait PackageId: Sized + Copy {
	fn from_name(name: &str) -> Option<Self>;
}

pub trait Registry {
	type PackageId: PackageId;
	type Resource;

	/// Returns the number of packages in the registry.
	fn len(&self) -> usize;

	/// Get the package with the given `id`.
	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>>;

	fn prelude_package_id(&self) -> Option<Self::PackageId>;

	fn is_empty(&self) -> bool {
		self.len() == 0
	}

	fn package_by_name(
		&self,
		name: Option<&str>,
	) -> Option<&Package<Self::PackageId, Self::Resource>> {
		self.package_id_from_name(name)
			.and_then(|id| self.package(id))
	}

	fn package_id_from_name(&self, name: Option<&str>) -> Option<Self::PackageId> {
		match name {
			Some(name) => PackageId::from_name(name),
			None => self.prelude_package_id(),
		}
	}

	fn prelude_package(&self) -> Option<&Package<Self::PackageId, Self::Resource>> {
		self.prelude_package_id().and_then(|id| self.package(id))
	}

	fn get_module(
		&self,
		id: ModuleRef<Self::PackageId>,
	) -> Option<&Module<Self::PackageId, Self::Resource>> {
		self.package(id.package)?.modules.get(id.index)
	}

	fn get_type(
		&self,
		id: TypeRef<Self::PackageId>,
	) -> Option<&TypeDefinition<Self::PackageId, Self::Resource>> {
		self.package(id.package)?.types.get(id.index)
	}

	fn get_function(
		&self,
		id: FunctionRef<Self::PackageId>,
	) -> Option<&Function<Self::PackageId, Self::Resource>> {
		self.package(id.package)?.functions.get(id.index)
	}

	fn get_by_name(&self, name: &str) -> Option<ItemRef<Self::PackageId>> {
		match self.package_by_name(Some(name)) {
			Some(p) => Some(ItemRef::Module(p.root_ref())),
			None => self.prelude_package()?.root().get_by_name(name),
		}
	}

	fn get(&self, path: &ItemPath) -> Option<ItemRef<Self::PackageId>> {
		let (first, rest) = path.split_first()?;

		match first {
			ItemPathSegment::Super => None,
			ItemPathSegment::Item(name) => self
				.get_by_name(name)
				.and_then(|i| i.get(self, ItemPath::from_slice(rest))),
		}
	}

	fn with_prelude_package(
		self,
		package: Package<Self::PackageId, Self::Resource>,
	) -> WithPreludePackage<Self>
	where
		Self: Sized,
	{
		WithPreludePackage::new(self, package)
	}
}

pub trait RegistryMut: Registry {
	fn insert_package(&mut self, package: Package<Self::PackageId, Self::Resource>);

	fn package_mut(
		&mut self,
		id: Self::PackageId,
	) -> Option<&mut Package<Self::PackageId, Self::Resource>>;

	fn insert_module(
		&mut self,
		id: Self::PackageId,
		m: Module<Self::PackageId, Self::Resource>,
	) -> ModuleRef<Self::PackageId> {
		let i = self.package_mut(id).unwrap().modules.insert(m);
		ModuleRef::new(id, i)
	}

	fn insert_type(
		&mut self,
		id: Self::PackageId,
		ty: impl Into<Type<Self::PackageId, Self::Resource>>,
	) -> TypeRef<Self::PackageId> {
		let i = self
			.package_mut(id)
			.unwrap()
			.types
			.insert(TypeDefinition::new(ty.into()));
		TypeRef::new(id, i)
	}

	fn insert_function(
		&mut self,
		id: Self::PackageId,
		f: Function<Self::PackageId, Self::Resource>,
	) -> FunctionRef<Self::PackageId> {
		let i = self.package_mut(id).unwrap().functions.insert(f);
		FunctionRef::new(id, i)
	}

	/// Compiles a package and inserts it in the registry.
	fn compile_and_insert_package_with(
		&mut self,
		rdf: &mut impl RdfContextMut<Resource = Self::Resource>,
		package_id: Self::PackageId,
		text: &str,
	) -> Result<(), CompileError<Self::PackageId>>
	where
		Self::PackageId: Clone + Ord + Hash,
		Self::Resource: Clone + Ord + Hash,
	{
		let package = Package::compile(rdf, self, package_id, text)?;
		self.insert_package(package);
		Ok(())
	}

	/// Compiles a package and inserts it in the registry, with the default RDF
	/// context.
	fn compile_and_insert_package(
		&mut self,
		package_id: Self::PackageId,
		text: &str,
	) -> Result<(), CompileError<Self::PackageId>>
	where
		Self::PackageId: Clone + Ord + Hash,
		Self: Registry<Resource = Term>,
	{
		let mut rdf = Rdf::default();
		let package = Package::compile(&mut rdf, self, package_id, text)?;
		self.insert_package(package);
		Ok(())
	}
}

impl<P: PackageId + Eq + Hash, R> Registry for HashMap<P, Package<P, R>> {
	type PackageId = P;
	type Resource = R;

	fn len(&self) -> usize {
		self.len()
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		self.get(&id)
	}

	fn prelude_package_id(&self) -> Option<Self::PackageId> {
		None
	}
}

impl<P: PackageId + Eq + Hash, R> RegistryMut for HashMap<P, Package<P, R>> {
	fn insert_package(&mut self, package: Package<Self::PackageId, Self::Resource>) {
		self.insert(package.id, package);
	}

	fn package_mut(
		&mut self,
		id: Self::PackageId,
	) -> Option<&mut Package<Self::PackageId, Self::Resource>> {
		self.get_mut(&id)
	}
}

pub struct EmptyRegistry<P, R>(PhantomData<(P, R)>);

impl<P, R> Default for EmptyRegistry<P, R> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<P, R> EmptyRegistry<P, R> {
	pub fn new() -> Self {
		Self::default()
	}
}

impl<P: PackageId, R> Registry for EmptyRegistry<P, R> {
	type PackageId = P;
	type Resource = R;

	fn len(&self) -> usize {
		0
	}

	fn package(&self, _id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		None
	}

	fn prelude_package_id(&self) -> Option<Self::PackageId> {
		None
	}
}

impl<T: ?Sized + Registry> Registry for &T {
	type PackageId = T::PackageId;

	type Resource = T::Resource;

	fn len(&self) -> usize {
		T::len(*self)
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		T::package(*self, id)
	}

	fn prelude_package_id(&self) -> Option<Self::PackageId> {
		T::prelude_package_id(*self)
	}
}

impl<T: ?Sized + Registry> Registry for &mut T {
	type PackageId = T::PackageId;

	type Resource = T::Resource;

	fn len(&self) -> usize {
		T::len(*self)
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		T::package(*self, id)
	}

	fn prelude_package_id(&self) -> Option<Self::PackageId> {
		T::prelude_package_id(*self)
	}
}

pub struct RegistryUnion<A, B>(pub A, pub B);

impl<A, B> Registry for RegistryUnion<A, B>
where
	A: Registry,
	B: Registry<PackageId = A::PackageId, Resource = A::Resource>,
{
	type PackageId = A::PackageId;
	type Resource = A::Resource;

	fn len(&self) -> usize {
		self.0.len() + self.1.len()
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		self.0.package(id).or_else(|| self.1.package(id))
	}

	fn prelude_package_id(&self) -> Option<Self::PackageId> {
		self.0
			.prelude_package_id()
			.or_else(|| self.1.prelude_package_id())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PreludeOrTarget {
	Prelude,
	Target,
}

impl PackageId for PreludeOrTarget {
	fn from_name(_: &str) -> Option<Self> {
		None
	}
}

pub struct PreludeAndTarget {
	pub prelude: Package<PreludeOrTarget>,
	pub target: Option<Package<PreludeOrTarget>>,
}

impl Default for PreludeAndTarget {
	fn default() -> Self {
		Self::new()
	}
}

impl PreludeAndTarget {
	pub fn new() -> Self {
		Self {
			prelude: Package::prelude(PreludeOrTarget::Prelude),
			target: None,
		}
	}
}

impl Registry for PreludeAndTarget {
	type PackageId = PreludeOrTarget;
	type Resource = rdf_types::Term;

	fn len(&self) -> usize {
		if self.target.is_some() {
			2
		} else {
			1
		}
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		match id {
			PreludeOrTarget::Prelude => Some(&self.prelude),
			PreludeOrTarget::Target => self.target.as_ref(),
		}
	}

	fn prelude_package_id(&self) -> Option<Self::PackageId> {
		Some(PreludeOrTarget::Prelude)
	}
}

impl RegistryMut for PreludeAndTarget {
	fn insert_package(&mut self, package: Package<Self::PackageId, Self::Resource>) {
		match package.id {
			PreludeOrTarget::Prelude => self.prelude = package,
			PreludeOrTarget::Target => self.target = Some(package),
		}
	}

	fn package_mut(
		&mut self,
		id: Self::PackageId,
	) -> Option<&mut Package<Self::PackageId, Self::Resource>> {
		match id {
			PreludeOrTarget::Prelude => Some(&mut self.prelude),
			PreludeOrTarget::Target => self.target.as_mut(),
		}
	}
}

pub struct WithPreludePackage<R: Registry> {
	pub registry: R,
	pub prelude: Package<R::PackageId, R::Resource>,
}

impl<R: Registry> WithPreludePackage<R> {
	pub fn new(registry: R, prelude: Package<R::PackageId, R::Resource>) -> Self {
		Self { registry, prelude }
	}
}

impl<R: Registry> Registry for WithPreludePackage<R>
where
	R::PackageId: PartialEq,
{
	type PackageId = R::PackageId;
	type Resource = R::Resource;

	fn len(&self) -> usize {
		self.registry.len() + 1
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		if id == self.prelude.id {
			Some(&self.prelude)
		} else {
			None
		}
	}

	fn prelude_package_id(&self) -> Option<Self::PackageId> {
		Some(self.prelude.id)
	}
}
