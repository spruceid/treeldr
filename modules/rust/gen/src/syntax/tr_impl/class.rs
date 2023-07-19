use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub struct AssociatedType {
	pub ident: Ident,
	pub lifetime: Option<syn::Lifetime>,
	pub value: syn::Type,
}

impl ToTokens for AssociatedType {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let lft1 = self.lifetime.iter();
		let lft2 = self.lifetime.iter();
		let value = &self.value;
		tokens.extend(quote!(type #ident #(<#lft1>)* = #value #(where Self: #lft2, I: #lft2)*;))
	}
}

pub struct TraitImpl {
	pub type_path: syn::Type,
	pub trait_path: syn::Path,
	pub context_bounds: Vec<syn::TraitBound>,
	pub associated_types: Vec<AssociatedType>,
	pub methods: Vec<Method>,
}

impl ToTokens for TraitImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ty_path = &self.type_path;
		let tr_path = &self.trait_path;
		// let context_bounds = &self.context_bounds;
		let associated_types = &self.associated_types;
		let methods = &self.methods;

		tokens.extend(quote! {
			impl<I> #tr_path for #ty_path {
				#(#associated_types)*
				#(#methods)*
			}
		})
	}
}

pub struct Method {
	pub ident: Ident,
	pub return_ty: syn::Type,
	pub body: TokenStream,
}

impl ToTokens for Method {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let return_ty = &self.return_ty;
		let body = &self.body;

		tokens.extend(quote! {
			fn #ident <'r> (&'r self) -> #return_ty where I: 'r {
				#body
			}
		})
	}
}
