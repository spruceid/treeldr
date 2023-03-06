use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use rdf_types::Vocabulary;
use treeldr::{vocab::Primitive, BlankIdIndex, IriIndex, TId};

use crate::{
	ty::{self, params::ParametersValues},
	Error, Generate, GenerateIn,
};

mod r#enum;
mod r#struct;

/// `RdfTriples` trait implementation.
pub struct RdfTriplesImpl;

/// Bound that may appear in a `RdfTriples` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bound {
	/// The given primitive must implement `AsLiteral`.
	AsLiteral(Primitive),
}

impl<M> Generate<M> for Bound {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		match self {
			Self::AsLiteral(p) => {
				let ty = p.generate_with(context, scope).into_tokens()?;
				tokens.extend(quote!(#ty: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>));
				Ok(())
			} // Self::TriplesAndValues(layout_ref) => {
			  // 	let ty = context.layout_type(*layout_ref).unwrap();
			  // 	let mut path = context.module_path(scope).to(&context.parent_module_path(ty.module()).unwrap());
			  // 	path.push(ty.ident());
			  // 	tokens.extend(quote!(#path: ::treeldr_rust_prelude::rdf::TriplesAndValues<N, V>));
			  // 	Ok(())
			  // }
		}
	}
}

/// Returns the name of the triples and values iterator derived from the given
/// layout name.
fn triples_and_values_iterator_name_from(ident: &Ident) -> Ident {
	format_ident!("{ident}TriplesAndValues")
}

/// Returns a path to the triples and values iterator of the given layout.
fn triples_and_values_iterator_of<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
	context: &crate::Context<V, M>,
	scope: Option<shelves::Ref<crate::Module>>,
	params_values: &ParametersValues,
	layout: TId<treeldr::Layout>,
	lifetime: TokenStream,
) -> Result<TokenStream, Error> {
	let ty = context.layout_type(layout).unwrap();

	match ty.description() {
		ty::Description::Never => Ok(quote!(::treeldr_rust_prelude::rdf::iter::Empty<V>)),
		ty::Description::Alias(a) => {
			triples_and_values_iterator_of(context, scope, params_values, a.target(), lifetime)
		}
		ty::Description::Primitive(p) => {
			let p_ty = p.generate_with(context, scope).into_tokens()?;
			Ok(
				quote!(::treeldr_rust_prelude::rdf::ValuesOnly<::treeldr_rust_prelude::rdf::LiteralValue<'a, #p_ty, I, V>>),
			)
		}
		ty::Description::BuiltIn(b) => match b {
			ty::BuiltIn::BTreeSet(item_layout) => {
				let item_ty_expr = item_layout
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				let inner = triples_and_values_iterator_of(
					context,
					scope,
					params_values,
					*item_layout,
					lifetime.clone(),
				)?;
				Ok(quote!(::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
						::std::collections::btree_set::Iter<#lifetime, #item_ty_expr>,
						#inner,
						V
					>))
			}
			ty::BuiltIn::OneOrMany(_) => {
				todo!()
			}
			ty::BuiltIn::Option(item_layout) => {
				let inner = triples_and_values_iterator_of(
					context,
					scope,
					params_values,
					*item_layout,
					lifetime,
				)?;
				Ok(quote!(::treeldr_rust_prelude::rdf::iter::Optional<#inner>))
			}
			ty::BuiltIn::Required(item_layout) => triples_and_values_iterator_of(
				context,
				scope,
				params_values,
				*item_layout,
				lifetime,
			),
			ty::BuiltIn::Vec(item_layout) => {
				let item_ty_expr = item_layout
					.generate_in_with(context, scope, params_values)
					.into_tokens()?;
				Ok(
					quote!(::treeldr_rust_prelude::rdf::iter::Flatten<::std::slice::Iter<#lifetime, #item_ty_expr>>),
				)
			}
		},
		ty::Description::Struct(s) => {
			let mut path = context
				.module_path(scope)
				.to(&context.parent_module_path(ty.module()).unwrap());
			path.push(triples_and_values_iterator_name_from(s.ident()));
			Ok(quote!(#path<#lifetime, I, V>))
		}
		ty::Description::Enum(e) => {
			let mut path = context
				.module_path(scope)
				.to(&context.parent_module_path(ty.module()).unwrap());
			path.push(triples_and_values_iterator_name_from(e.ident()));
			Ok(quote!(#path<#lifetime, I, V>))
		}
		ty::Description::IdentifierParameter => Ok(quote!(
			::treeldr_rust_prelude::rdf::ValuesOnly<::treeldr_rust_prelude::rdf::IdValue<'a, I, V>>
		)),
	}
}

/// Collect the bounds necessary for the `Triples` or `TriplesAndValues`
/// implementations.
fn collect_bounds<V, M>(
	context: &crate::Context<V, M>,
	layout: TId<treeldr::Layout>,
	mut bound: impl FnMut(Bound),
) {
	fn collect<V, M>(
		context: &crate::Context<V, M>,
		layout: TId<treeldr::Layout>,
		bound: &mut impl FnMut(Bound),
	) {
		let ty = context.layout_type(layout).unwrap();

		match ty.description() {
			ty::Description::Never => (),
			ty::Description::Alias(a) => collect(context, a.target(), bound),
			ty::Description::Primitive(p) => bound(Bound::AsLiteral(*p)),
			ty::Description::BuiltIn(b) => match b {
				ty::BuiltIn::BTreeSet(item_layout) => collect(context, *item_layout, bound),
				ty::BuiltIn::OneOrMany(item_layout) => collect(context, *item_layout, bound),
				ty::BuiltIn::Option(item_layout) => collect(context, *item_layout, bound),
				ty::BuiltIn::Required(item_layout) => collect(context, *item_layout, bound),
				ty::BuiltIn::Vec(item_layout) => collect(context, *item_layout, bound),
			},
			ty::Description::Struct(s) => {
				for field in s.fields() {
					collect(context, field.layout(), bound)
				}
			}
			ty::Description::Enum(e) => {
				for variant in e.variants() {
					if let Some(layout_ref) = variant.ty() {
						collect(context, layout_ref, bound)
					}
				}
			}
			ty::Description::IdentifierParameter => (),
		}
	}

	collect(context, layout, &mut bound)
}
