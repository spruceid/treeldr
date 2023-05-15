use rdf_types::Vocabulary;
use thiserror::Error;

pub use shelves::Ref;

mod context;
mod error;
pub mod fmt;
pub mod module;
pub mod path;
pub mod syntax;
pub mod tr;
pub mod ty;

pub use context::{Context, Options};
pub use error::Error;
pub use module::Module;
pub use path::Path;
use treeldr::{BlankIdIndex, IriIndex};
pub use ty::Type;

#[derive(Clone, Default)]
pub struct BoundParameters {
	pub identifier: Option<syn::Type>,
	pub context: Option<syn::Type>
}

impl BoundParameters {
	pub fn get(&self, p: crate::ty::Parameter) -> Option<&syn::Type> {
		match p {
			crate::ty::Parameter::Identifier => self.identifier.as_ref(),
			crate::ty::Parameter::Context => self.context.as_ref(),
		}
	}
}

#[derive(Clone)]
pub struct Scope<'a> {
	pub module: Option<Ref<Module>>,
	pub params: BoundParameters,
	pub self_trait: Option<&'a tr::Trait>
}

impl<'a> Scope<'a> {
	pub fn new(module: Option<Ref<Module>>) -> Self {
		Self {
			module,
			params: BoundParameters::default(),
			self_trait: None
		}
	}

	pub fn bound_params(&self) -> &BoundParameters {
		&self.params
	}
}

pub trait GenerateSyntax<M> {
	type Output;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error>;
}

pub struct Referenced<T>(T);
