use proc_macro2::Ident;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

use crate::{syntax, Context, Error, GenerateSyntax};

use super::{AssociatedType, AssociatedTypeBound, Method, MethodType, Trait};

impl<M> GenerateSyntax<M> for Trait {
	type Output = syntax::ClassTraitDefinition;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.context = Some(syn::Type::Path(syn::TypePath {
			qself: None,
			path: Ident::new("C", proc_macro2::Span::call_site()).into(),
		}));
		scope.self_trait = Some(self);

		let dyn_table_path = context
			.module_path(scope.module)
			.to(&self.dyn_table_path(context).unwrap())
			.generate_syntax(context, &scope)?;

		let mut super_traits = Vec::new();
		super_traits.push(
			syn::parse2(quote!(::treeldr_rust_prelude::AsTraitObject<#dyn_table_path>)).unwrap(),
		);
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
			associated_types.push(ty.generate_syntax(context, &scope)?);
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
	type Output = syntax::TraitAssociatedType;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let never_value = match self.bound() {
			AssociatedTypeBound::Types(_) => {
				// associated_types_trait_objects.push(associated_trait_object_type(
				// 	context,
				// 	scope,
				// 	ty.trait_object_ident(self).unwrap(),
				// 	classes,
				// )?);
				syn::parse2(quote!(&'r ::std::convert::Infallible)).unwrap()
			}
			AssociatedTypeBound::Collection(_) => {
				syn::parse2(quote!(::std::iter::Empty<&'r ::std::convert::Infallible>)).unwrap()
			}
		};

		let ident = self.ident();
		Ok(syntax::TraitAssociatedType {
			ident: self.ident().clone(),
			bounds: self.bound().generate_syntax(context, scope)?,
			never_value,
			ref_value: syn::parse2(quote!(T::#ident<'r>)).unwrap(),
		})
	}
}

impl<M> GenerateSyntax<M> for AssociatedTypeBound {
	type Output = Vec<syn::TypeParamBound>;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut result = Vec::new();

		match self {
			Self::Types(refs) => {
				result.push(syn::parse2(quote!(::treeldr_rust_prelude::Reference<'r>)).unwrap());

				for type_ref in refs {
					result.push(syn::TypeParamBound::Trait(syn::TraitBound {
						paren_token: None,
						modifier: syn::TraitBoundModifier::None,
						lifetimes: None,
						path: type_ref.generate_syntax(context, scope)?,
					}))
				}
			}
			Self::Collection(i) => {
				result.push(syn::TypeParamBound::Lifetime(syn::Lifetime {
					apostrophe: proc_macro2::Span::call_site(),
					ident: Ident::new("r", proc_macro2::Span::call_site()),
				}));

				let a_ident = scope.self_trait.unwrap().associated_types[*i].ident();
				result.push(syn::TypeParamBound::Trait(syn::TraitBound {
					paren_token: None,
					modifier: syn::TraitBoundModifier::None,
					lifetimes: None,
					path: syn::parse2(quote!(Iterator<Item = Self::#a_ident <'r>>)).unwrap(),
				}))
			}
		}

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
			ref_body: quote!(T::#ident(*self, context)),
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
				let ty_ident = a.ident();
				Ok(syn::parse2(quote!(Self::#ty_ident<'r>)).unwrap())
			}
			Self::Option(i) => {
				let a = &tr.associated_types[*i];
				let ty_ident = a.ident();
				Ok(syn::parse2(quote!(Option<Self::#ty_ident<'r>>)).unwrap())
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
