use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use crate::{
	function::FunctionRef, ty::TypeDefinition, Function, Module, ModuleRef, Package, Type, TypeRef,
};

pub trait Registry {
	type PackageId: Copy;
	type Resource;

	/// Returns the number of packages in the registry.
	fn len(&self) -> usize;

	fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Get the package with the given `id`.
	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>>;

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
}

impl<P: Copy + Eq + Hash, R> Registry for HashMap<P, Package<P, R>> {
	type PackageId = P;
	type Resource = R;

	fn len(&self) -> usize {
		self.len()
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		self.get(&id)
	}
}

impl<P: Copy + Eq + Hash, R> RegistryMut for HashMap<P, Package<P, R>> {
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

impl<P: Copy, R> Registry for EmptyRegistry<P, R> {
	type PackageId = P;
	type Resource = R;

	fn len(&self) -> usize {
		0
	}

	fn package(&self, _id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		None
	}
}

impl<'a, T: Registry> Registry for &'a T {
	type PackageId = T::PackageId;

	type Resource = T::Resource;

	fn len(&self) -> usize {
		T::len(*self)
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		T::package(*self, id)
	}
}

impl<'a, T: Registry> Registry for &'a mut T {
	type PackageId = T::PackageId;

	type Resource = T::Resource;

	fn len(&self) -> usize {
		T::len(*self)
	}

	fn package(&self, id: Self::PackageId) -> Option<&Package<Self::PackageId, Self::Resource>> {
		T::package(*self, id)
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
}
