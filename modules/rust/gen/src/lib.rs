use quote::ToTokens;
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

pub use context::{Context, DedicatedSubModule, ModulePathBuilder, Options};
pub use error::Error;
pub use module::Module;
pub use path::Path;
use treeldr::{BlankIdIndex, IriIndex};
pub use ty::Type;

pub enum GenericArgumentRef<'a> {
	Lifetime(&'a syn::Lifetime),
	Type(&'a syn::Type),
}

impl<'a> GenericArgumentRef<'a> {
	pub fn into_owned(self) -> syn::GenericArgument {
		match self {
			Self::Lifetime(l) => syn::GenericArgument::Lifetime(l.clone()),
			Self::Type(t) => syn::GenericArgument::Type(t.clone()),
		}
	}
}

impl<'a> ToTokens for GenericArgumentRef<'a> {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::Lifetime(l) => l.to_tokens(tokens),
			Self::Type(t) => t.to_tokens(tokens),
		}
	}
}

#[derive(Clone, Default)]
pub struct BoundParameters {
	pub lifetime: Option<syn::Lifetime>,
	pub identifier: Option<syn::Type>,
	pub context: Option<syn::Type>,
}

impl BoundParameters {
	pub fn get(&self, p: crate::ty::Parameter) -> Option<GenericArgumentRef> {
		match p {
			crate::ty::Parameter::Lifetime => {
				self.lifetime.as_ref().map(GenericArgumentRef::Lifetime)
			}
			crate::ty::Parameter::Identifier => {
				self.identifier.as_ref().map(GenericArgumentRef::Type)
			}
			crate::ty::Parameter::Context => self.context.as_ref().map(GenericArgumentRef::Type),
		}
	}
}

#[derive(Clone)]
pub struct Scope<'a> {
	pub module: Option<Ref<Module>>,
	pub params: BoundParameters,
	pub self_trait: Option<&'a tr::Trait>,
}

impl<'a> Scope<'a> {
	pub fn new(module: Option<Ref<Module>>) -> Self {
		Self {
			module,
			params: BoundParameters::default(),
			self_trait: None,
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
