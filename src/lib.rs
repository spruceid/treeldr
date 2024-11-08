pub mod attributes;
pub mod display;
pub mod eval;
pub mod expr;
pub mod function;
pub mod layout;
pub mod matching;
pub mod pattern;
mod prelude;
pub mod rdf;
pub mod registry;
pub mod syntax;
pub mod ty;
pub mod utils;
pub mod value;

use core::fmt;
use std::{
	borrow::Cow,
	collections::{BTreeMap, HashMap},
	fmt::Write,
	hash::Hash,
	ops::Deref,
};

pub use attributes::{Attribute, Attributes};
use display::DisplaySignature;
use educe::Educe;
pub use function::Function;
use function::FunctionRef;
pub use layout::Layout;
pub use matching::Matching;
pub use pattern::{DatasetPattern, Graph, TermPattern};
use rdf::RdfContextMut;
pub use registry::Registry;
pub use ty::{Type, TypeRef};
pub use value::{Literal, Value};

#[derive(Debug, Clone, Educe)]
#[educe(Default)]
pub struct Module<R = rdf_types::Term, T = TypeRef> {
	pub attributes: Attributes<R>,
	pub items: BTreeMap<String, Item<R, T>>,
}

impl<R, T> Module<R, T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, path: &ItemPath) -> Option<ItemRef<R, T>>
	where
		T: Clone,
	{
		match path.split_first() {
			Some((first, [])) => self.get_by_name(first),
			Some((first, rest)) => match self.get_by_name(first)? {
				ItemRef::Module(m) => m.get(ItemPath::from_slice(rest)),
				_ => None,
			},
			None => Some(ItemRef::Module(self)),
		}
	}

	pub fn get_by_name(&self, name: &str) -> Option<ItemRef<R, T>>
	where
		T: Clone,
	{
		Some(self.items.get(name)?.as_ref())
	}

	pub fn insert(&mut self, name: impl Into<String>, item: Item<R, T>) -> Option<Item<R, T>> {
		self.items.insert(name.into(), item)
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Module<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Module<R, U> {
		Module {
			attributes: self.attributes,
			items: self
				.items
				.into_iter()
				.map(|(name, i)| (name, i.map_types_ref_mut(f)))
				.collect(),
		}
	}

	// pub fn display(&self) -> Disp
}

impl<R: Clone + Ord + Hash> Module<R> {
	/// Compiles a module.
	pub fn compile(
		rdf: &mut impl RdfContextMut<R>,
		registry: &mut Registry<R>,
		env: &Environment<R>,
		text: &str,
	) -> Result<Self, CompileError> {
		use syntax::{
			build::{self, Build, BuildContext},
			Parse,
		};
		let ast = syntax::ast::Module::parse_str(text)?;

		let mut context = BuildContext::new(rdf, registry);

		let scope = build::Scope::Root(env);
		let module = ast.build(&mut context, &scope)?;

		let unresolved = module.build(&mut context, &build::Scope::Module(&scope, &module))?;

		let resolution = context.resolve()?;

		Ok(unresolved.map_types(|x| resolution[&x]))
	}
}

impl<R: fmt::Display> DisplaySignature<R> for Module<R> {
	fn fmt_signature(
		&self,
		context: &display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		for (name, item) in &self.items {
			match item {
				Item::Type(item) => writeln!(
					f,
					"{}type {} = {}",
					context.indent,
					name,
					item.display_signature_with(context.inner())
				)?,
				Item::Function(item) => {
					let function = context.registry.get_function(*item).unwrap();

					write!(f, "{}let ({}", context.indent, name)?;

					for a in &function.signature.args {
						write!(f, " {}", a.display_signature_with(context.inner()))?
					}

					writeln!(
						f,
						"): {}",
						function
							.signature
							.return_type
							.display_signature_with(context.inner())
					)?
				}
				Item::Module(m) => {
					writeln!(f, "{}mod {} {{", context.indent, name)?;
					m.fmt_signature(&context.inner(), f)?;
					writeln!(f, "{}}}", context.indent)?
				}
			}
		}

		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
	#[error(transparent)]
	Parse(#[from] syntax::ParseError),

	#[error(transparent)]
	Build(#[from] syntax::build::Error),

	#[error(transparent)]
	Typing(#[from] syntax::build::TypeResolutionError),
}

#[derive(Debug, Clone)]
pub enum Item<R, T = TypeRef> {
	Type(T),
	Function(FunctionRef),
	Module(Module<R, T>),
}

impl<R, T> Item<R, T> {
	pub fn as_ref(&self) -> ItemRef<R, T>
	where
		T: Clone,
	{
		match self {
			Self::Type(t) => ItemRef::Type(t.clone()),
			Self::Function(f) => ItemRef::Function(*f),
			Self::Module(m) => ItemRef::Module(m),
		}
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Item<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Item<R, U> {
		match self {
			Self::Type(t) => Item::Type(f(t)),
			Self::Function(f) => Item::Function(f),
			Self::Module(m) => Item::Module(m.map_types_ref_mut(f)),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ItemRef<'a, R, T = TypeRef> {
	Type(T),
	Function(FunctionRef),
	Module(&'a Module<R, T>),
}

#[derive(Debug)]
pub struct ItemPath(pub [String]);

impl ItemPath {
	pub fn from_slice(slice: &[String]) -> &Self {
		unsafe { std::mem::transmute(slice) }
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

pub struct Environment<'a, R: Clone = rdf_types::Term> {
	prelude: Option<Cow<'a, Module<R>>>,
	imports: HashMap<String, Cow<'a, Module<R>>>,
}

impl<'a, R: Clone> Default for Environment<'a, R> {
	fn default() -> Self {
		Self::new(None)
	}
}

impl<'a, R: Clone> Environment<'a, R> {
	pub fn new(prelude: Option<Cow<'a, Module<R>>>) -> Self {
		Self {
			prelude,
			imports: HashMap::new(),
		}
	}

	pub fn get(&self, path: &ItemPath) -> Option<ItemRef<R>> {
		let (first, rest) = path.split_first()?;
		match self.get_by_name(first)? {
			ItemRef::Module(m) => m.get(ItemPath::from_slice(rest)),
			_ => None,
		}
	}

	pub fn get_by_name(&self, name: &str) -> Option<ItemRef<R>> {
		match self.imports.get(name) {
			Some(m) => Some(ItemRef::Module(m)),
			None => self.prelude.as_ref()?.get_by_name(name),
		}
	}
}
