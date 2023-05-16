use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};

pub struct TraitImpl {
	pub type_path: syn::Type,
	pub trait_path: syn::Path,
	pub context_bounds: Vec<syn::TraitBound>,
	pub associated_types: Vec<(Ident, syn::Type)>,
	pub methods: Vec<Method>,
	pub dyn_table_path: syn::Path,
	pub dyn_table_instance_path: syn::Path
}

impl ToTokens for TraitImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ty_path = &self.type_path;
		let tr_path = &self.trait_path;
		let context_bounds = &self.context_bounds;
		let associated_types = self.associated_types.iter().map(|(id, ty)| {
			quote!(type #id <'a> = #ty where Self: 'a , C: 'a ;)
		});
		let methods = &self.methods;
		let dyn_table_path = &self.dyn_table_path;
		let dyn_table_instance_path = &self.dyn_table_instance_path;

		tokens.extend(quote! {
			impl <C: ?Sized #(+#context_bounds)*> #tr_path for #ty_path <C> {
				#(#associated_types)*
				#(#methods)*
			}

			unsafe impl <C: ?Sized #(+#context_bounds)*> ::treeldr_rust_prelude::AsTraitObject<#dyn_table_path<C>> for #ty_path <C> {
				fn as_trait_object(&self) -> (*const u8, #dyn_table_instance_path<C>) {
					let table = #dyn_table_instance_path::new::<Self>();
					(self as *const Self as *const u8, table)
				}
			}
		})
	}
}

pub struct Method {
	pub ident: Ident,
	pub return_ty: syn::Type,
	pub body: TokenStream
}

impl ToTokens for Method {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let return_ty = &self.return_ty;
		let body = &self.body;

		tokens.extend(quote! {
			fn #ident <'a> (&'a self, context: &'a C) -> #return_ty {
				#body
			}
		})
	}
}