use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

pub struct Derived {
	pub ident: Ident,
	pub base: syn::Type,
	pub restrictions: Vec<syn::Expr>,
	pub default_value: Option<TokenStream>,
}

impl ToTokens for Derived {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let base = &self.base;
		let checks = &self.restrictions;

		let error_ident = format_ident!("Invalid{}", ident);

		tokens.extend(quote! {
			#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
			pub struct #ident(#base);

			impl #ident {
				pub fn into_base(self) -> #base {
					self.0
				}

				pub fn as_base(&self) -> &#base {
					&self.0
				}
			}

			impl ::std::fmt::Display for #ident {
				fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
					self.0.fmt(f)
				}
			}

			impl ::std::ops::Deref for #ident {
				type Target = #base;

				fn deref(&self) -> &Self::Target {
					&self.0
				}
			}
		});

		if self.restrictions.is_empty() {
			tokens.extend(quote! {
				impl #ident {
					pub fn new(value: #base) -> Self {
						Self(value)
					}
				}

				impl From<#base> for #ident {
					fn from(value: #base) -> Self {
						Self::new(value)
					}
				}
			})
		} else {
			tokens.extend(quote! {
				#[derive(Debug)]
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

				impl TryFrom<#base> for #ident {
					type Error = #error_ident;

					fn try_from(value: #base) -> Result<Self, Self::Error> {
						Self::new(value)
					}
				}
			})
		}

		if let Some(value) = self.default_value.as_ref() {
			tokens.extend(quote! {
				impl Default for #ident {
					fn default() -> Self {
						Self(#value)
					}
				}
			})
		}
	}
}
