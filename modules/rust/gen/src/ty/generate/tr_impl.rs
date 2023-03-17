use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use shelves::Ref;
use treeldr::TId;

use crate::{
	module::{TraitId, TraitImpl},
	tr::ContextBound,
	ty::{
		self,
		params::{Parameter, ParametersValues}
	},
	Context, Error, Generate, GenerateIn, Module,
};

use super::GenerateFor;

mod primitive;
mod r#struct;
mod r#enum;

impl<M> GenerateIn<M> for ContextBound {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let tr = context.type_trait(self.0).unwrap();
		context
			.module_path(scope)
			.to(&tr
				.context_path(context)
				.ok_or(Error::UnreachableTrait(self.0))?)
			.to_tokens(tokens);
		let id_param_value = params_values.get(Parameter::Identifier);
		tokens.extend(quote! { <#id_param_value> });
		Ok(())
	}
}

fn context_bounds_tokens<
	V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	M,
>(
	bounds: &BTreeSet<ContextBound>,
	context: &Context<V, M>,
	scope: Option<Ref<Module>>,
	params: &ParametersValues,
) -> Result<TokenStream, Error> {
	let mut tokens = quote!(?Sized);

	for b in bounds {
		tokens.extend(quote!(+));
		b.generate_in(context, scope, params, &mut tokens)?
	}

	Ok(tokens)
}

impl<M> Generate<M> for TraitImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ty = context.layout_type(self.ty).unwrap();

		match ty.description() {
			ty::Description::Struct(s) => match self.tr {
				TraitId::FromRdf => {
					super::rdf::from::FromRdfImpl.generate(context, scope, s, tokens)
				}
				TraitId::TriplesAndValues => {
					super::rdf::to::RdfTriplesImpl.generate(context, scope, s, tokens)
				}
				TraitId::IntoJsonLd => {
					super::json_ld::IntoJsonLdImpl.generate(context, scope, s, tokens)
				}
				TraitId::Defined(tr) => ClassTraitImpl(tr).generate(context, scope, s, tokens),
			},
			ty::Description::Enum(e) => match self.tr {
				TraitId::FromRdf => {
					super::rdf::from::FromRdfImpl.generate(context, scope, e, tokens)
				}
				TraitId::TriplesAndValues => {
					super::rdf::to::RdfTriplesImpl.generate(context, scope, e, tokens)
				}
				TraitId::IntoJsonLd => {
					super::json_ld::IntoJsonLdImpl.generate(context, scope, e, tokens)
				}
				TraitId::Defined(tr) => ClassTraitImpl(tr).generate(context, scope, e, tokens),
			},
			ty::Description::Primitive(p) => match self.tr {
				TraitId::Defined(tr) => ClassTraitImpl(tr).generate(context, scope, p, tokens),
				_ => Ok(()),
			},
			_ => {
				panic!("unable to implement trait for non enum/struct type")
			}
		}
	}
}

/// Class trait implementation.
pub struct ClassTraitImpl(TId<treeldr::Type>);

fn collection_iterator<V, M>(
	context: &Context<V, M>,
	scope: Option<shelves::Ref<crate::Module>>,
	collection_layout: TId<treeldr::Layout>,
	params_values: &ParametersValues,
) -> Result<TokenStream, Error>
where
	V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
{
	let ty = context.layout_type(collection_layout).unwrap();
	match ty.description() {
		ty::Description::BuiltIn(b) => match b {
			ty::BuiltIn::Vec(item) => {
				let item_expr = item
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(quote!(::std::slice::Iter<'a, #item_expr>))
			}
			ty::BuiltIn::Option(item) => {
				let item_expr = item
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(quote!(::std::option::Iter<'a, #item_expr>))
			}
			ty::BuiltIn::BTreeSet(item) => {
				let item_expr = item
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(quote!(::std::collections::btree_set::Iter<'a, #item_expr>))
			}
			ty::BuiltIn::OneOrMany(item) => {
				let item_expr = item
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(quote!(::treeldr_rust_prelude::one_or_many::Iter<'a, #item_expr>))
			}
			ty::BuiltIn::Required(_) => panic!("cannot turn required layout into iterator"),
		},
		_ => panic!("cannot turn a non-built-in layout into an iterator"),
	}
}