use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};

pub struct Restricted {
	pub ident: Ident,
	pub base: syn::Type,
	pub restrictions: Vec<syn::Expr>,
}

impl ToTokens for Restricted {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let base = &self.base;
		let checks = &self.restrictions;

		let error_ident = format_ident!("Invalid{}", ident);

		tokens.extend(quote! {
			pub struct #ident(#base);

			pub struct #error_ident(pub #base);

			impl #ident {
				pub fn new(value: #base) -> Result<Self, #error_ident> {
					if Self::check(&value) {
						Ok(Self(value))
					} else {
						Err(#error_ident(value))
					}
				}

				fn check(value: &#base) -> bool {
					#(#checks)&&*
				}
			}
		})
	}
}
