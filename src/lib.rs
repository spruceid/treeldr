use std::collections::HashMap;

pub mod attributes;
pub mod build;
pub mod display;
pub mod eval;
pub mod expr;
pub mod function;
pub mod layout;
pub mod matching;
pub mod module;
pub mod pattern;
mod prelude;
pub mod rdf;
pub mod registry;
pub mod syntax;
pub mod ty;
pub mod utils;
pub mod value;

pub use attributes::{Attribute, Attributes};
pub use function::Function;
pub use layout::Layout;
pub use matching::Matching;
use module::{ItemPath, ItemPathSegment, ItemRef};
pub use module::{Module, ModuleRef};
pub use pattern::{DatasetPattern, Graph, TermPattern};
pub use registry::Registry;
pub use ty::{Type, TypeRef};
pub use value::{Literal, Value};

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
