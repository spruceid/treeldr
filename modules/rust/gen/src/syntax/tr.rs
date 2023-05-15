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

pub enum TraitImplementation {
	ClassTraitImpl(ClassTraitImpl)
}

impl ToTokens for TraitImplementation {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::ClassTraitImpl(i) => i.to_tokens(tokens)
		}
	}
}

pub struct ClassTraitImpl {
	ident: Ident,
	trait_path: syn::Path,
	context_bounds: Vec<syn::TraitBound>,
	associated_types: Vec<(Ident, syn::Type)>,
	methods: Vec<ClassTraitImplMethod>,
	dyn_table_path: syn::Path,
	dyn_table_instance_path: syn::Path
}

impl ToTokens for ClassTraitImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let tr_path = &self.trait_path;
		let context_bounds = &self.context_bounds;
		let associated_types = self.associated_types.iter().map(|(id, ty)| {
			quote!(type #id <'a> = #ty where Self: 'a , C: 'a ;)
		});
		let methods = &self.methods;
		let dyn_table_path = &self.dyn_table_path;
		let dyn_table_instance_path = &self.dyn_table_instance_path;

		tokens.extend(quote! {
			impl <C: ?Sized #(+#context_bounds)*> #tr_path for #ident <C> {
				#(#associated_types)*
				#(#methods)*
			}

			unsafe impl <C: ?Sized #(+#context_bounds)*> ::treeldr_rust_prelude::AsTraitObject<#dyn_table_path<C>> for #ident <C> {
				fn as_trait_object(&self) -> (*const u8, #dyn_table_instance_path<C>) {
					let table = #dyn_table_instance_path::new::<Self>();
					(self as *const Self as *const u8, table)
				}
			}
		})
	}
}

pub struct ClassTraitImplMethod {
	// ...
}

impl ToTokens for ClassTraitImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		todo!()
	}
}

pub struct ClassTraitDefinition {
	pub ident: Ident,
	pub super_traits: Vec<syn::TraitBound>,
	pub associated_types: Vec<TraitAssociatedType>,
	pub methods: Vec<TraitMethod>
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
			quote!(type #t_ident <'a> = #value where Self: 'a, C: 'a;)
		});

		let never_methods = self.methods.iter().map(|m| {
			let m_ident = &m.ident;
			let return_type = &m.return_type;
			let body = &m.never_body;

			quote!(fn #m_ident <'a> (&'a self, context: &'a C) -> #return_type {
				#body
			})
		});

		let ref_associated_types = self.associated_types.iter().map(|t| {
			let t_ident = &t.ident;
			let value = &t.ref_value;
			quote!(type #t_ident <'a> = #value where Self: 'a, C: 'a;)
		});

		let ref_methods = self.methods.iter().map(|m| {
			let m_ident = &m.ident;
			let return_type = &m.return_type;
			let body = &m.ref_body;

			quote!(fn #m_ident <'a> (&'a self, context: &'a C) -> #return_type {
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

			impl<'r, C: ?Sized, T: #ident<C>> #ident <C> for &'r T {
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
	pub ref_value: syn::Type
}

impl ToTokens for TraitAssociatedType {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let bounds = &self.bounds;

		tokens.extend(quote! {
			type #ident <'a> : #(#bounds)+* where Self: 'a, C: 'a;
		})
	}
}

pub struct TraitMethod {
	pub ident: Ident,
	pub return_type: syn::Type,
	pub never_body: TokenStream,
	pub ref_body: TokenStream
}

impl ToTokens for TraitMethod {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let ty = &self.return_type;

		tokens.extend(quote! {
			fn #ident <'a> (&'a self, context: &'a C) -> #ty;
		})
	}
}

pub struct ClassProviderTraitDefinition {
	pub ident: Ident,
	pub class_ident: Ident,
	pub trait_path: syn::Path
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