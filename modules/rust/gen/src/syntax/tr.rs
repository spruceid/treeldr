use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub enum TraitDefinition {
	Class(ClassTraitDefinition),
	ClassProvider(ClassProviderTraitDefinition),
}

impl ToTokens for TraitDefinition {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::Class(c) => c.to_tokens(tokens),
			Self::ClassProvider(c) => c.to_tokens(tokens),
		}
	}
}

macro_rules! class_trait_params_type {
	{ $( $name:ident ),* } => {
		pub struct ClassTraitParameters {
			$( pub $name : Option<Ident> ),*
		}

		impl ClassTraitParameters {
			pub fn is_empty(&self) -> bool {
				$( self.$name.is_none() )&&*
			}

			pub fn iter(&self) -> ClassTraitParametersIter {
				ClassTraitParametersIter {
					$( $name: self.$name.as_ref() ),*
				}
			}
		}

		pub struct ClassTraitParametersIter<'a> {
			$( $name : Option<&'a Ident> ),*
		}

		impl<'a> Iterator for ClassTraitParametersIter<'a> {
			type Item = &'a Ident;

			fn next(&mut self) -> Option<Self::Item> {
				$(
					if let Some(ty) = self.$name.take() {
						return Some(ty)
					}
				)*

				None
			}
		}
	};
}

class_trait_params_type! {
	context
}

impl ToTokens for ClassTraitParameters {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		if !self.is_empty() {
			let types = self.iter();
			tokens.extend(quote!(< #(#types),* >))
		}
	}
}

pub struct ClassTraitDefinition {
	pub ident: Ident,
	pub super_traits: Vec<syn::TraitBound>,
	pub associated_types: Vec<TraitAssociatedType>,
	pub methods: Vec<TraitMethod>,
}

impl ToTokens for ClassTraitDefinition {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let super_traits = &self.super_traits;
		let associated_types = &self.associated_types;
		let methods = &self.methods;

		let never_associated_types = self.associated_types.iter().map(|t| {
			let t_ident = &t.ident;
			let value = &t.never_value;
			quote!(type #t_ident <'r> = #value where Self: 'r, C: 'r;)
		});

		let never_methods = self.methods.iter().map(|m| {
			let m_ident = &m.ident;
			let return_type = &m.return_type;
			let body = &m.never_body;

			quote!(fn #m_ident <'r> (&'r self, context: &'r C) -> #return_type {
				#body
			})
		});

		let ref_associated_types = self.associated_types.iter().map(|t| {
			let t_ident = &t.ident;
			let value = &t.ref_value;
			quote!(type #t_ident <'r> = #value where Self: 'r, C: 'r;)
		});

		let ref_methods = self.methods.iter().map(|m| {
			let m_ident = &m.ident;
			let return_type = &m.return_type;
			let body = &m.ref_body;

			quote!(fn #m_ident <'r> (&'r self, context: &'r C) -> #return_type {
				#body
			})
		});

		tokens.extend(quote! {
			pub trait #ident <C: ?Sized> : #(#super_traits)+* {
				#(#associated_types)*
				#(#methods)*
			}

			impl <C: ?Sized> #ident <C> for ::std::convert::Infallible {
				#(#never_associated_types)*
				#(#never_methods)*
			}

			impl<'d, C: ?Sized, T: #ident<C>> #ident <C> for &'d T {
				#(#ref_associated_types)*
				#(#ref_methods)*
			}
		})
	}
}

pub struct TraitAssociatedType {
	pub ident: Ident,
	pub bounds: Vec<syn::TypeParamBound>,
	pub never_value: syn::Type,
	pub ref_value: syn::Type,
}

impl ToTokens for TraitAssociatedType {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let bounds = &self.bounds;

		tokens.extend(quote! {
			type #ident <'r> : #(#bounds)+* where Self: 'r, C: 'r;
		})
	}
}

pub struct TraitMethod {
	pub ident: Ident,
	pub return_type: syn::Type,
	pub never_body: TokenStream,
	pub ref_body: TokenStream,
}

impl ToTokens for TraitMethod {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let ty = &self.return_type;

		tokens.extend(quote! {
			fn #ident <'r> (&'r self, context: &'r C) -> #ty;
		})
	}
}

pub struct ClassProviderTraitDefinition {
	pub ident: Ident,
	pub class_ident: Ident,
	pub trait_path: syn::Path,
}

impl ToTokens for ClassProviderTraitDefinition {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let class_ident = &self.class_ident;
		let tr_path = &self.trait_path;

		tokens.extend(quote! {
			pub trait #ident <I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::#class_ident> {
				type #class_ident: #tr_path;

				fn get(&self, id: &I) -> Option<&Self::#class_ident> {
					<Self as ::treeldr_rust_prelude::Provider<I, Self::#class_ident>>::get(self, id)
				}
			}
		})
	}
}
