use core::fmt;
use std::{
	collections::{BTreeMap, HashMap},
	fmt::Write,
	hash::Hash,
	ops::Deref,
};

pub mod attributes;
pub mod build;
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

pub use attributes::{Attribute, Attributes};
use display::DisplaySignature;
use educe::Educe;
pub use function::Function;
use function::FunctionRef;
use iref::{Iri, IriBuf};
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
	pub parent: Option<ModuleRef>,
	pub attributes: Attributes<R>,
	pub items: BTreeMap<String, Item<T>>,
}

impl<R, T> Module<R, T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get_by_segment(&self, segment: &ItemPathSegment) -> Option<ItemRef<T>>
	where
		T: Clone,
	{
		match segment {
			ItemPathSegment::Super => self.parent.map(ItemRef::Module),
			ItemPathSegment::Item(name) => self.get_by_name(name),
		}
	}

	pub fn get_by_name(&self, name: &str) -> Option<ItemRef<T>>
	where
		T: Clone,
	{
		Some(self.items.get(name)?.as_ref())
	}

	pub fn insert(&mut self, name: impl Into<String>, item: Item<T>) -> Option<Item<T>> {
		self.items.insert(name.into(), item)
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Module<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Module<R, U> {
		Module {
			parent: self.parent,
			attributes: self.attributes,
			items: self
				.items
				.into_iter()
				.map(|(name, i)| (name, i.map_types_ref_mut(f)))
				.collect(),
		}
	}
}

impl<R> Module<R> {
	pub fn get<'a>(
		&'a self,
		registry: &'a Registry<R>,
		this: ModuleRef,
		path: &ItemPath,
	) -> Option<ItemRef<'a>> {
		match path.split_first() {
			Some((first, rest)) => self
				.get_by_segment(first)
				.and_then(|i| i.get(registry, ItemPath::from_slice(rest))),
			None => Some(ItemRef::Module(this)),
		}
	}
}

impl<R: Clone + Ord + Hash> Module<R> {
	/// Compiles a module.
	pub fn compile(
		rdf: &mut impl RdfContextMut<R>,
		registry: &mut Registry<R>,
		env: &Environment,
		text: &str,
	) -> Result<ModuleRef, CompileError> {
		use build::{self, BuildContext, BuildingNamespace};
		use syntax::Parse;

		let ast = syntax::ast::Module::parse_str(text)?;

		let mut namespace = BuildingNamespace::new(registry.create_namespace());
		let mut context = BuildContext::new(rdf, registry, env);

		let scope = build::Scope::new();

		let module_ref = ast.declare(&mut context, &mut namespace, &scope, None)?;

		namespace.build(&mut context)?.resolve(context)?;

		Ok(module_ref)
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
				Item::Prefix(item) => writeln!(f, "{}prefix = <{item}>", context.indent)?,
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
	Build(#[from] build::Error),

	#[error(transparent)]
	Typing(#[from] build::TypeResolutionError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleRef {
	namespace: u32,
	index: u32,
}

impl ModuleRef {
	pub fn new(namespace: u32, index: u32) -> Self {
		Self { namespace, index }
	}

	pub fn get<'a, R>(&self, registry: &'a Registry<R>, path: &ItemPath) -> Option<ItemRef<'a>> {
		registry.get_module(*self)?.get(registry, *self, path)
	}

	pub fn get_by_segment<'a, R>(
		&self,
		registry: &'a Registry<R>,
		segment: &ItemPathSegment,
	) -> Option<ItemRef<'a>> {
		registry.get_module(*self)?.get_by_segment(segment)
	}

	pub fn get_by_name<'a, R>(&self, registry: &'a Registry<R>, name: &str) -> Option<ItemRef<'a>> {
		registry.get_module(*self)?.get_by_name(name)
	}
}

impl<R: fmt::Display> DisplaySignature<R> for ModuleRef {
	fn fmt_signature(
		&self,
		context: &display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		context
			.registry
			.get_module(*self)
			.unwrap()
			.fmt_signature(context, f)
	}
}

#[derive(Debug, Clone)]
pub enum Item<T = TypeRef> {
	Prefix(IriBuf),
	Type(T),
	Function(FunctionRef),
	Module(ModuleRef),
}

impl<T> Item<T> {
	pub fn as_ref(&self) -> ItemRef<T>
	where
		T: Clone,
	{
		match self {
			Self::Prefix(i) => ItemRef::Prefix(i),
			Self::Type(t) => ItemRef::Type(t.clone()),
			Self::Function(f) => ItemRef::Function(*f),
			Self::Module(m) => ItemRef::Module(*m),
		}
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Item<U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Item<U> {
		match self {
			Self::Prefix(i) => Item::Prefix(i),
			Self::Type(t) => Item::Type(f(t)),
			Self::Function(f) => Item::Function(f),
			Self::Module(m) => Item::Module(m),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ItemRef<'a, T = TypeRef> {
	Prefix(&'a Iri),
	Type(T),
	Function(FunctionRef),
	Module(ModuleRef),
}

impl<'a, T> ItemRef<'a, T> {
	pub fn as_function(&self) -> Option<FunctionRef> {
		match self {
			Self::Function(f) => Some(*f),
			_ => None,
		}
	}

	pub fn as_module(&self) -> Option<ModuleRef> {
		match self {
			Self::Module(m) => Some(*m),
			_ => None,
		}
	}
}

impl<'a> ItemRef<'a> {
	pub fn get<R>(&self, registry: &'a Registry<R>, path: &ItemPath) -> Option<Self> {
		if path.is_empty() {
			Some(*self)
		} else {
			self.as_module()?.get(registry, path)
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemPathSegment {
	Super,
	Item(String),
}

impl fmt::Display for ItemPathSegment {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Super => write!(f, "super"),
			Self::Item(name) => f.write_str(name),
		}
	}
}

#[derive(Debug)]
pub struct ItemPath(pub [ItemPathSegment]);

impl ItemPath {
	pub fn from_slice(slice: &[ItemPathSegment]) -> &Self {
		unsafe { std::mem::transmute(slice) }
	}
}

impl Deref for ItemPath {
	type Target = [ItemPathSegment];

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
pub struct ItemPathBuf(pub Vec<ItemPathSegment>);

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

pub struct Environment {
	prelude: Option<ModuleRef>,
	imports: HashMap<String, ModuleRef>,
}

impl Default for Environment {
	fn default() -> Self {
		Self::new(None)
	}
}

impl Environment {
	pub fn new(prelude: Option<ModuleRef>) -> Self {
		Self {
			prelude,
			imports: HashMap::new(),
		}
	}

	pub fn get<'a, R>(&self, registry: &'a Registry<R>, path: &ItemPath) -> Option<ItemRef<'a>> {
		let (first, rest) = path.split_first()?;

		match first {
			ItemPathSegment::Super => None,
			ItemPathSegment::Item(name) => self
				.get_by_name(registry, name)
				.and_then(|i| i.get(registry, ItemPath::from_slice(rest))),
		}
	}

	pub fn get_by_name<'a, R>(&self, registry: &'a Registry<R>, name: &str) -> Option<ItemRef<'a>> {
		match self.imports.get(name) {
			Some(m) => Some(ItemRef::Module(*m)),
			None => self.prelude.as_ref()?.get_by_name(registry, name),
		}
	}
}
