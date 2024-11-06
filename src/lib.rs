pub mod rdf;
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
pub mod registry;
mod prelude;

use core::fmt;
use std::{borrow::Cow, collections::{BTreeMap, HashMap}, fmt::Write, hash::Hash, ops::Deref};

use educe::Educe;
use rdf::RdfContextMut;
pub use function::Function;
use function::FunctionRef;
pub use layout::Layout;
pub use matching::Matching;
pub use pattern::{DatasetPattern, Graph, TermPattern};
pub use ty::{Type, TypeRef};
pub use value::{Literal, Value};
pub use registry::Registry;

#[derive(Debug, Clone, Educe)]
#[educe(Default)]
pub struct Module<T = TypeRef> {
	items: BTreeMap<String, Item<T>>,
}

impl<T> Module<T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, path: &ItemPath) -> Option<ItemRef<T>> where T: Clone {
		match path.split_first() {
			Some((first, [])) => self.get_by_name(first),
			Some((first, rest)) => {
				match self.get_by_name(first)? {
					ItemRef::Module(m) => m.get(ItemPath::from_slice(rest)),
					_ => None
				}
			}
			None => Some(ItemRef::Module(self))
		}
	}
	
	pub fn get_by_name(&self, name: &str) -> Option<ItemRef<T>> where T: Clone {
		Some(self.items.get(name)?.as_ref())
	}

	pub fn insert(&mut self, name: impl Into<String>, item: Item<T>) -> Option<Item<T>> {
		self.items.insert(name.into(), item)
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Module<U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Module<U> {
		Module {
			items: self.items.into_iter().map(|(name, i)| (name, i.map_types_ref_mut(f))).collect()
		}
	}
}

impl Module {
	/// Compiles a module.
	pub fn compile<R: Clone + Ord + Hash>(
		rdf: &mut impl RdfContextMut<R>,
		registry: &mut Registry<R>,
		env: &Environment,
		text: &str
	) -> Result<Self, CompileError> {
		use syntax::{Parse, build::{self, Build, BuildContext}};
		let ast = syntax::ast::Module::parse_str(text)?;
	
		let mut context = BuildContext::new(
			rdf,
			registry
		);

		let scope = build::Scope::Root(env);
		let module = ast.build(
			&mut context,
			&scope
		)?;
	
		let unresolved = module.build(
			&mut context,
			&build::Scope::Module(&scope, &module)
		)?;

		let resolution = context
			.resolve()?;

		Ok(unresolved.map_types(|x| resolution[&x]))
	}
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
	#[error(transparent)]
	Parse(#[from] syntax::ParseError),

	#[error(transparent)]
	Build(#[from] syntax::build::Error),

	#[error(transparent)]
	Typing(#[from] syntax::build::TypeResolutionError)
}

#[derive(Debug, Clone)]
pub enum Item<T = TypeRef> {
	Type(T),
	Function(FunctionRef),
	Module(Module<T>),
}

impl<T> Item<T> {
	pub fn as_ref(&self) -> ItemRef<T> where T: Clone {
		match self {
			Self::Type(t) => ItemRef::Type(t.clone()),
			Self::Function(f) => ItemRef::Function(*f),
			Self::Module(m) => ItemRef::Module(m)
		}
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Item<U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Item<U> {
		match self {
			Self::Type(t) => Item::Type(f(t)),
			Self::Function(f) => Item::Function(f),
			Self::Module(m) => Item::Module(m.map_types_ref_mut(f))
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ItemRef<'a, T = TypeRef> {
	Type(T),
	Function(FunctionRef),
	Module(&'a Module<T>)
}

#[derive(Debug)]
pub struct ItemPath(pub [String]);

impl ItemPath {
	pub fn from_slice(slice: &[String]) -> &Self {
		unsafe {
			std::mem::transmute(slice)
		}
	}
}

impl Deref for ItemPath {
	type Target = [String];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl fmt::Display for ItemPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_empty() {
			write!(f, "self")
		} else {
			for (i, s) in self.iter().enumerate() {
				if i > 0 {
					f.write_char('.')?;
				}

				s.fmt(f)?;
			}

			Ok(())
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemPathBuf(pub Vec<String>);

impl Deref for ItemPathBuf {
	type Target = ItemPath;

	fn deref(&self) -> &Self::Target {
		ItemPath::from_slice(&self.0)
	}
}

impl fmt::Display for ItemPathBuf {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		ItemPath::from_slice(&self.0).fmt(f)
	}
}

pub struct Environment<'a> {
	prelude: Option<Cow<'a, Module>>,
	imports: HashMap<String, Cow<'a, Module>>
}

impl<'a> Default for Environment<'a> {
	fn default() -> Self {
		Self::new(None)
	}
}

impl<'a> Environment<'a> {
	pub fn new(prelude: Option<Cow<'a, Module>>) -> Self {
		Self {
			prelude,
			imports: HashMap::new()
		}
	}

	pub fn get(&self, path: &ItemPath) -> Option<ItemRef> {
		let (first, rest) = path.split_first()?;
		match self.get_by_name(first)? {
			ItemRef::Module(m) => m.get(ItemPath::from_slice(rest)),
			_ => None
		}
	}

	pub fn get_by_name(&self, name: &str) -> Option<ItemRef> {
		match self.imports.get(name) {
			Some(m) => Some(ItemRef::Module(m)),
			None => {
				self.prelude.as_ref()?.get_by_name(name)
			}
		}
	}
}