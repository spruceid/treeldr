use contextual::WithContext;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use treeldr::TId;

use crate::{
	ty::params::{Parameters, ParametersBounds, ParametersValues},
	Error, Generate, GenerateIn,
};

use super::{AssociatedType, AssociatedTypeBound, Method, MethodType, Trait};

impl Trait {
	pub fn generate_associated_type_expr(&self, ty_expr: TokenStream, index: usize) -> TokenStream {
		let ident = &self.associated_types[index].ident;
		quote!(#ty_expr :: #ident)
	}
}

impl<M> Generate<M> for Trait {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = &self.ident;
		let params_values = ParametersValues::new_for_trait(quote!(C));
		let params_bounds = ParametersBounds::new_for_trait(quote!(?Sized));
		let params = Parameters::context_parameter()
			.instantiate(&params_values)
			.with_bounds(&params_bounds);

		let mut super_traits = TokenStream::new();
		for &ty_ref in &self.super_traits {
			if super_traits.is_empty() {
				super_traits.extend(quote!(:))
			} else {
				super_traits.extend(quote!(+))
			}

			super_traits.extend(
				ty_ref
					.with(self)
					.generate_in_with(context, scope, &params_values)
					.into_tokens()?,
			)
		}

		let mut associated_types = Vec::with_capacity(self.associated_types.len());
		let mut never_associated_types = Vec::with_capacity(self.associated_types.len());
		let mut ref_associated_types = Vec::with_capacity(self.associated_types.len());
		for ty in &self.associated_types {
			associated_types.push(
				ty.with(self)
					.generate_in_with(context, scope, &params_values)
					.into_tokens()?,
			);

			let a_ident = ty.ident();
			let never_expr = match ty.bound() {
				AssociatedTypeBound::Types(_) => {
					quote!(::std::convert::Infallible)
				}
				AssociatedTypeBound::Collection(_) => {
					quote!(::std::iter::Empty<::std::convert::Infallible>)
				}
			};
			never_associated_types.push(quote! {
				type #a_ident <'a> = #never_expr where Self: 'a, C: 'a;
			});
			ref_associated_types.push(quote! {
				type #a_ident <'a> = T::#a_ident<'a> where Self: 'a, C: 'a;
			})
		}

		let mut methods = Vec::with_capacity(self.methods.len());
		let mut never_methods = Vec::with_capacity(self.methods.len());
		let mut ref_methods = Vec::with_capacity(self.methods.len());
		for m in &self.methods {
			methods.push(m.with(self).generate_with(context, scope).into_tokens()?);

			let m_ident = m.ident();
			let return_ty = match m.type_() {
				MethodType::Required(i) => {
					let ty_ident = self.associated_types[*i].ident();
					quote!(Self::#ty_ident<'a>)
				}
				MethodType::Option(i) => {
					let ty_ident = self.associated_types[*i].ident();
					quote!(Option<Self::#ty_ident<'a>>)
				}
			};
			never_methods.push(quote! {
				fn #m_ident <'a> (&'a self, _context: &'a C) -> #return_ty {
					unreachable!()
				}
			});
			ref_methods.push(quote! {
				fn #m_ident <'a> (&'a self, context: &'a C) -> #return_ty {
					T::#m_ident(*self, context)
				}
			})
		}

		let context_ident = self.context_ident();

		tokens.extend(quote! {
			pub trait #ident #params #super_traits {
				#(#associated_types)*

				#(#methods)*
			}

			pub trait #context_ident <I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::#ident> {
				type #ident: #ident <Self>;

				fn get(&self, id: &I) -> Option<&Self::#ident> {
					<Self as ::treeldr_rust_prelude::Provider<I, Self::#ident>>::get(self, id)
				}
			}

			impl <C: ?Sized> #ident <C> for ::std::convert::Infallible {
				#(#never_associated_types)*
				#(#never_methods)*
			}

			impl<'r, C: ?Sized, T: #ident<C>> #ident <C> for &'r T {
				#(#ref_associated_types)*
				#(#ref_methods)*
			}
		});

		Ok(())
	}
}

impl<'a, 't, M> Generate<M> for contextual::Contextual<&'a Method, &'t Trait> {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = &self.ident;
		let ty = self
			.ty
			.with(self.1)
			.generate_with(context, scope)
			.into_tokens()?;

		tokens.extend(quote! {
			fn #ident <'a> (&'a self, context: &'a C) -> #ty;
		});

		Ok(())
	}
}

impl<'a, 't, M> Generate<M> for contextual::Contextual<&'a MethodType, &'t Trait> {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		_context: &crate::Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		match self.0 {
			MethodType::Required(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(ty_expr);
				tokens.extend(quote!(<'a>));
			}
			MethodType::Option(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(quote!(Option<#ty_expr<'a>>))
			}
		}

		Ok(())
	}
}

impl<'a, 't, M> GenerateIn<M> for contextual::Contextual<&'a AssociatedType, &'t Trait> {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_values: &ParametersValues,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = &self.ident;
		let bound = self
			.bound
			.with(self.1)
			.generate_in_with(context, scope, params_values)
			.into_tokens()?;

		tokens.extend(quote! {
			type #ident <'a> : #bound;
		});

		Ok(())
	}
}

impl<'a, 't, M> GenerateIn<M> for contextual::Contextual<&'a AssociatedTypeBound, &'t Trait> {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_value: &ParametersValues,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		match self.0 {
			AssociatedTypeBound::Types(refs) => {
				tokens.extend(quote!('a));

				for type_ref in refs {
					tokens.extend(quote!(+));
					type_ref.generate_in(context, scope, params_value, tokens)?;
				}

				tokens.extend(quote!(where Self: 'a, C: 'a));

				Ok(())
			}
			AssociatedTypeBound::Collection(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(quote!('a + Iterator<Item = #ty_expr<'a>> where Self: 'a, C: 'a));
				Ok(())
			}
		}
	}
}

impl<M> GenerateIn<M> for TId<treeldr::Type> {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let tr = context.type_trait(*self).expect("trait not found");
		let path = tr.path(context).ok_or(Error::UnreachableTrait(*self))?;
		context.module_path(scope).to(&path).to_tokens(tokens);
		Parameters::context_parameter()
			.instantiate(params_values)
			.to_tokens(tokens);
		Ok(())
	}
}
