pub mod eval;
pub mod expr;
pub mod function;
pub mod layout;
pub mod matching;
pub mod pattern;
pub mod syntax;
pub mod ty;
pub mod utils;
pub mod value;

use std::{collections::BTreeMap, sync::Arc};

use educe::Educe;
pub use function::Function;
use function::FunctionRef;
pub use layout::Layout;
pub use matching::Matching;
pub use pattern::{DatasetPattern, Graph, TermPattern};
use ty::TypeDefinition;
pub use ty::{Type, TypeRef};
pub use value::{Literal, Value};

pub trait Domain {
	type Resource;
}

/// Registry.
pub struct Registry<R> {
	/// Types.
	types: Vec<TypeDefinition<R>>,

	/// Functions.
	functions: Vec<Function<R>>,
}

impl<R> Registry<R> {
	/// Returns the type definition referenced by `type_ref`.
	pub fn get_type(&self, type_ref: TypeRef) -> Option<&TypeDefinition<R>> {
		let i: usize = type_ref.into();
		self.types.get(i)
	}

	/// Returns the function definition referenced by `function_ref`.
	pub fn get_function(&self, function_ref: FunctionRef) -> Option<&Function<R>> {
		let i: usize = function_ref.into();
		self.functions.get(i)
	}
}

struct RegistryReservation<'a, R> {
	registry: &'a mut Registry<R>,
	types: Vec<Option<Type<R>>>,
	functions: Vec<Option<Function<R>>>,
}

impl<'a, R> RegistryReservation<'a, R> {
	pub fn new(registry: &'a mut Registry<R>) -> Self {
		Self {
			registry,
			types: Vec::new(),
			functions: Vec::new(),
		}
	}

	pub fn reserve_type(&mut self) -> TypeRef {
		todo!()
	}

	pub fn reserve_function(&mut self) -> FunctionRef {
		todo!()
	}

	pub fn define_type(&mut self, r: TypeRef, value: Type<R>) {
		todo!()
	}

	pub fn define_function(&mut self, r: FunctionRef, value: Function<R>) {
		todo!()
	}
}

#[derive(Debug, Educe)]
#[educe(Default)]
pub struct Module<T = TypeRef> {
	items: BTreeMap<String, Item<T>>,
}

impl<T> Module<T> {
	pub fn insert(&mut self, name: impl Into<String>, item: Item<T>) -> Option<Item<T>> {
		self.items.insert(name.into(), item)
	}
}

#[derive(Debug)]
pub enum Item<T = TypeRef> {
	Type(T),
	Function(FunctionRef),
	Module(Module<T>),
}
