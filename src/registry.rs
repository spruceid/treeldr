use educe::Educe;

use crate::{function::FunctionRef, ty::TypeDefinition, Function, Type, TypeRef};

/// Registry.
#[derive(Debug, Educe)]
#[educe(Default)]
pub struct Registry<R = rdf_types::Term> {
	namespaces: Vec<Namespace<R>>,
}

impl<R> Registry<R> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn namespace_count(&self) -> u32 {
		self.namespaces.len() as u32
	}

	pub fn create_namespace(&mut self) -> u32 {
		let i = self.namespace_count();
		self.namespaces.push(Namespace::default());
		i
	}

	pub fn get_namespace(&self, i: u32) -> Option<&Namespace<R>> {
		self.namespaces.get(i as usize)
	}

	pub fn get_namespace_mut(&mut self, i: u32) -> Option<&mut Namespace<R>> {
		self.namespaces.get_mut(i as usize)
	}

	/// Returns the type definition referenced by `type_ref`.
	pub fn get_type(&self, type_ref: TypeRef) -> Option<&TypeDefinition<R>> {
		self.get_namespace(type_ref.namespace)?
			.get_type(type_ref.index)
	}

	/// Returns the function definition referenced by `function_ref`.
	pub fn get_function(&self, function_ref: FunctionRef) -> Option<&Function<R>> {
		self.get_namespace(function_ref.namespace)?
			.get_function(function_ref.index)
	}

	pub fn insert_type(&mut self, namespace: u32, type_: impl Into<Type<R>>) -> TypeRef {
		let index = self
			.get_namespace_mut(namespace)
			.unwrap()
			.insert_type(type_.into());
		TypeRef::new(namespace, index)
	}

	pub fn insert_function(&mut self, namespace: u32, function: Function<R>) -> FunctionRef {
		let index = self
			.get_namespace_mut(namespace)
			.unwrap()
			.insert_function(function);
		FunctionRef::new(namespace, index)
	}
}

#[derive(Debug, Educe)]
#[educe(Default)]
pub struct Namespace<R> {
	/// Types.
	types: Vec<TypeDefinition<R>>,

	/// Functions.
	functions: Vec<Function<R>>,
}

impl<R> Namespace<R> {
	pub fn get_type(&self, i: u32) -> Option<&TypeDefinition<R>> {
		self.types.get(i as usize)
	}

	pub fn get_function(&self, i: u32) -> Option<&Function<R>> {
		self.functions.get(i as usize)
	}

	pub fn insert_type(&mut self, type_: Type<R>) -> u32 {
		let i = self.types.len() as u32;
		self.types.push(TypeDefinition::new(type_));
		i
	}

	pub fn insert_function(&mut self, function: Function<R>) -> u32 {
		let i = self.functions.len() as u32;
		self.functions.push(function);
		i
	}
}

pub struct TypeAllocator<'a, R> {
	registry: &'a mut Registry<R>,
	namespace: u32,
	reserved_types: Vec<Option<Type<R>>>,
}

impl<'a, R> TypeAllocator<'a, R> {
	pub fn new(registry: &'a mut Registry<R>, namespace: u32) -> Self {
		Self {
			registry,
			namespace,
			reserved_types: Vec::new(),
		}
	}

	pub fn reserve_type(&mut self) -> TypeRef {
		let namespace = self.registry.get_namespace(self.namespace).unwrap();
		let ty_ref = TypeRef::new(
			self.namespace,
			(namespace.types.len() + self.reserved_types.len()) as u32,
		);
		self.reserved_types.push(None);
		ty_ref
	}

	pub fn define_type(&mut self, r: TypeRef, value: Type<R>) {
		let namespace = self.registry.get_namespace(self.namespace).unwrap();
		let i = r.index as usize - namespace.types.len();
		self.reserved_types[i] = Some(value);
	}
}

impl<'a, R> Drop for TypeAllocator<'a, R> {
	fn drop(&mut self) {
		let namespace = self.registry.get_namespace_mut(self.namespace).unwrap();
		for t in self.reserved_types.drain(..) {
			match t {
				Some(t) => {
					namespace.insert_type(t);
				}
				None => return,
			}
		}
	}
}
