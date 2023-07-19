use proc_macro2::Ident;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

use crate::{syntax, Context, Error, GenerateSyntax};

use super::{
	AssociatedCollectionTypeBound, AssociatedPropertyTypeBounds, AssociatedType, Method,
	MethodType, Trait,
};

impl<M> GenerateSyntax<M> for Trait {
	type Output = syntax::ClassTraitDefinition;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(I)).unwrap());
		scope.params.context = Some(syn::parse2(quote!(C)).unwrap());
		scope.self_trait = Some(self);

		let mut super_traits = Vec::new();
		for &ty_ref in &self.super_traits {
			super_traits.push(syn::TraitBound {
				paren_token: None,
				modifier: syn::TraitBoundModifier::None,
				lifetimes: None,
				path: ty_ref.generate_syntax(context, &scope)?,
			})
		}

		let mut associated_types = Vec::with_capacity(self.associated_types.len());
		for ty in &self.associated_types {
			associated_types.extend(ty.generate_syntax(context, &scope)?);
		}

		let mut methods = Vec::with_capacity(self.methods.len());
		for m in &self.methods {
			methods.push(m.generate_syntax(context, &scope)?);
		}

		Ok(syntax::ClassTraitDefinition {
			ident: self.ident.clone(),
			super_traits,
			associated_types,
			methods,
		})
	}
}

impl<M> GenerateSyntax<M> for AssociatedType {
	type Output = Vec<syntax::TraitAssociatedType>;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut result = Vec::with_capacity(3);

		let ident = self.ident();
		result.push(syntax::TraitAssociatedType {
			ident: ident.clone(),
			lifetime: None,
			bounds: self.bounds().generate_syntax(context, scope)?,
			never_value: syn::parse2(quote!(::std::convert::Infallible)).unwrap(),
			ref_value: syn::parse2(quote!(T::#ident)).unwrap(),
		});

		if let Some(collection_ident) = &self.collection_ident {
			result.push(syntax::TraitAssociatedType {
				ident: collection_ident.clone(),
				lifetime: Some(syn::parse2(quote!('r)).unwrap()),
				bounds: self.collection_bound().generate_syntax(context, scope)?,
				never_value: syn::parse2(quote!(
					::std::iter::Empty<
						::treeldr_rust_prelude::Ref<'r, I, ::std::convert::Infallible>,
					>
				))
				.unwrap(),
				ref_value: syn::parse2(quote!(T::#collection_ident<'r>)).unwrap(),
			});
		}

		Ok(result)
	}
}

impl<M> GenerateSyntax<M> for AssociatedPropertyTypeBounds {
	type Output = Vec<syn::TypeParamBound>;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut result = Vec::new();

		for tr_ref in self.traits() {
			result.push(syn::TypeParamBound::Trait(syn::TraitBound {
				paren_token: None,
				modifier: syn::TraitBoundModifier::None,
				lifetimes: None,
				path: tr_ref.generate_syntax(context, scope)?,
			}))
		}

		Ok(result)
	}
}

impl<'a, M> GenerateSyntax<M> for AssociatedCollectionTypeBound<'a> {
	type Output = Vec<syn::TypeParamBound>;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		_scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut result = Vec::new();

		result.push(syn::TypeParamBound::Lifetime(syn::Lifetime {
			apostrophe: proc_macro2::Span::call_site(),
			ident: Ident::new("r", proc_macro2::Span::call_site()),
		}));

		let a_ident = self.item_ident();
		result.push(syn::TypeParamBound::Trait(syn::TraitBound {
			paren_token: None,
			modifier: syn::TraitBoundModifier::None,
			lifetimes: None,
			path: syn::parse2(
				quote!(Iterator<Item = ::treeldr_rust_prelude::Ref<'r, I, Self::#a_ident>>),
			)
			.unwrap(),
		}));

		Ok(result)
	}
}

impl<M> GenerateSyntax<M> for Method {
	type Output = syntax::TraitMethod;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let ident = &self.ident;
		Ok(syntax::TraitMethod {
			ident: self.ident().clone(),
			return_type: self.ty.generate_syntax(context, scope)?,
			never_body: quote!(unreachable!()),
			ref_body: quote!(T::#ident(*self)),
		})
	}
}

impl<M> GenerateSyntax<M> for MethodType {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let tr = scope.self_trait.unwrap();

		match self {
			Self::Required(i) => {
				let a = &tr.associated_types[*i];
				match a.collection_ident() {
					Some(ident) => Ok(syn::parse2(quote!(Self::#ident <'r>)).unwrap()),
					None => {
						let ident = a.ident();
						Ok(syn::parse2(quote!(&'r Self::#ident)).unwrap())
					}
				}
			}
			Self::Option(i) => {
				let a = &tr.associated_types[*i];
				match a.collection_ident() {
					Some(ident) => Ok(syn::parse2(quote!(Option<Self::#ident<'r>>)).unwrap()),
					None => {
						let ident = a.ident();
						Ok(syn::parse2(quote!(Option<&'r Self::#ident>)).unwrap())
					}
				}
			}
		}
	}
}

impl<M> GenerateSyntax<M> for TId<treeldr::Type> {
	type Output = syn::Path;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let tr = context.type_trait(*self).expect("trait not found");
		let path = tr
			.path(context)
			.ok_or_else(|| Error::unreachable_trait(*self))?;
		context
			.module_path(scope.module)
			.to(&path)
			.generate_syntax(context, scope)
	}
}
