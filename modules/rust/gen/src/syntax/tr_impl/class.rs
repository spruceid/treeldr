use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub struct TraitImpl {
	pub type_path: syn::Type,
	pub type_params: Vec<syn::TypeParam>,
	pub trait_path: syn::Path,
	pub context_bounds: Vec<syn::TraitBound>,
	pub associated_types: Vec<(Ident, syn::Type)>,
	pub methods: Vec<Method>,
	pub dyn_table_path: syn::Path,
	pub dyn_table_instance_path: syn::Path,
}

impl ToTokens for TraitImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ty_path = &self.type_path;
		let ty_params = &self.type_params;
		let tr_path = &self.trait_path;
		let context_bounds = &self.context_bounds;
		let associated_types = self
			.associated_types
			.iter()
			.map(|(id, ty)| quote!(type #id <'r> = #ty where Self: 'r , C: 'r ;));
		let methods = &self.methods;
		let dyn_table_path = &self.dyn_table_path;
		let dyn_table_instance_path = &self.dyn_table_instance_path;

		tokens.extend(quote! {
			impl <#(#ty_params,)* C: ?Sized #(+#context_bounds)*> #tr_path for #ty_path {
				#(#associated_types)*
				#(#methods)*
			}

			unsafe impl <#(#ty_params,)* C: ?Sized #(+#context_bounds)*> ::treeldr_rust_prelude::AsTraitObject<#dyn_table_path> for #ty_path {
				fn as_trait_object<'r>(&'r self) -> (*const u8, #dyn_table_instance_path) {
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
	pub body: TokenStream,
}

impl ToTokens for Method {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let return_ty = &self.return_ty;
		let body = &self.body;

		tokens.extend(quote! {
			fn #ident <'r> (&'r self, context: &'r C) -> #return_ty {
				#body
			}
		})
	}
}