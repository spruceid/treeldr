use proc_macro2::Ident;
use quote::{quote, ToTokens};

use crate::module::ExternPath;

pub mod ty;
pub mod tr;
pub mod tr_impl;

pub use ty::*;
pub use tr::*;
pub use tr_impl::TraitImplementation;

pub enum ModuleOrUse {
	Module(Module),
	Use(Use),
}

impl ToTokens for ModuleOrUse {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::Module(m) => m.to_tokens(tokens),
			Self::Use(u) => u.to_tokens(tokens),
		}
	}
}

pub struct Use {
	pub vis: syn::Visibility,
	pub path: ExternPath,
	pub ident: Option<Ident>,
}

impl ToTokens for Use {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let vis = &self.vis;
		let path = &self.path;

		match self.ident.as_ref() {
			Some(ident) => tokens.extend(quote! { #vis use #path as #ident ; }),
			None => tokens.extend(quote! { #vis use #path ; }),
		}
	}
}

pub struct Module {
	pub vis: syn::Visibility,
	pub ident: Ident,
	pub content: ModuleContent,
}

impl ToTokens for Module {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let vis = &self.vis;
		let ident = &self.ident;
		let content = &self.content;

		tokens.extend(quote! {
			#vis mod #ident {
				#content
			}
		})
	}
}

/// Module.
pub struct ModuleContent {
	pub items: Vec<ModuleItem>,
}

impl ToTokens for ModuleContent {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for item in &self.items {
			item.to_tokens(tokens)
		}
	}
}

pub enum ModuleItem {
	Module(ModuleOrUse),
	Trait(TraitDefinition),
	Type(TypeDefinition),
	TraitImpl(TraitImplementation),
}

impl ToTokens for ModuleItem {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::Module(m) => m.to_tokens(tokens),
			Self::Trait(t) => t.to_tokens(tokens),
			Self::Type(t) => t.to_tokens(tokens),
			Self::TraitImpl(t) => t.to_tokens(tokens),
		}
	}
}

