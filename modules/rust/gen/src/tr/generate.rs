use contextual::WithContext;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use treeldr::TId;

use crate::{Error, Generate};

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
					.generate_with(context, scope)
					.into_tokens()?,
			)
		}

		let mut associated_types = Vec::with_capacity(self.associated_types.len());
		for ty in &self.associated_types {
			associated_types.push(ty.with(self).generate_with(context, scope).into_tokens()?)
		}

		let mut methods = Vec::with_capacity(self.methods.len());
		for m in &self.methods {
			methods.push(m.with(self).generate_with(context, scope).into_tokens()?)
		}

		tokens.extend(quote! {
			pub trait #ident #super_traits {
				#(#associated_types)*

				#(#methods)*
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
			fn #ident (&self) -> #ty;
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
			MethodType::Direct(i, has_lifetime) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(ty_expr);
				if *has_lifetime {
					tokens.extend(quote!(<'_>));
				}
			}
			MethodType::Reference(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(quote!(&#ty_expr))
			}
			MethodType::Option(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(quote!(Option<&#ty_expr>))
			}
		}

		Ok(())
	}
}

impl<'a, 't, M> Generate<M> for contextual::Contextual<&'a AssociatedType, &'t Trait> {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = &self.ident;
		let bound = self
			.bound
			.with(self.1)
			.generate_with(context, scope)
			.into_tokens()?;

		let params = if self.has_lifetime {
			Some(quote!(<'a>))
		} else {
			None
		};

		tokens.extend(quote! {
			type #ident #params : #bound;
		});

		Ok(())
	}
}

impl<'a, 't, M> Generate<M> for contextual::Contextual<&'a AssociatedTypeBound, &'t Trait> {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		match self.0 {
			AssociatedTypeBound::Types(refs) => {
				for (i, type_ref) in refs.iter().enumerate() {
					if i > 0 {
						tokens.extend(quote!(+));
					}

					type_ref.generate(context, scope, tokens)?
				}

				Ok(())
			}
			AssociatedTypeBound::Collection(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(quote!(Iterator<Item = &'a #ty_expr> where Self: 'a));
				Ok(())
			}
		}
	}
}

impl<M> Generate<M> for TId<treeldr::Type> {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let tr = context.type_trait(*self).expect("trait not found");
		let path = tr.path(context).ok_or(Error::UnreachableTrait(*self))?;
		context.module_path(scope).to(&path).to_tokens(tokens);
		Ok(())
	}
}
