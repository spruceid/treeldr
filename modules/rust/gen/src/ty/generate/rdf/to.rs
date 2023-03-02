use proc_macro2::{Ident, TokenStream};
use quote::{quote, format_ident};
use rdf_types::Vocabulary;
use treeldr::{TId, IriIndex, BlankIdIndex, vocab::Primitive};

use crate::{ty, Generate, Error};

mod r#struct;
mod r#enum;

/// `RdfTriples` trait implementation.
pub struct RdfTriplesImpl;

/// Bound that may appear in a `RdfTriples` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bound {
	/// The given primitive must implement `AsLiteral`.
	AsLiteral(Primitive),
	TriplesAndValues(TId<treeldr::Layout>)
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
			}
			Self::TriplesAndValues(layout_ref) => {
				let ty = context.layout_type(*layout_ref).unwrap();
				let mut path = context.module_path(scope).to(&context.parent_module_path(ty.module()).unwrap());
				path.push(ty.ident());
				tokens.extend(quote!(#path: ::treeldr_rust_prelude::rdf::TriplesAndValues<N, V>));
				Ok(())
			}
		}
	}
}

/// Returns the name of the triples and values iterator derived from the given
/// layout name.
fn triples_and_values_iterator_name_from(
	ident: &Ident
) -> Ident {
	format_ident!("{ident}TriplesAndValues")
}

/// Returns a path to the triples and values iterator of the given layout.
fn triples_and_values_iterator_of<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
	context: &crate::Context<V, M>,
	scope: Option<shelves::Ref<crate::Module>>,
	layout: TId<treeldr::Layout>,
	lifetime: TokenStream
) -> Result<(TokenStream, Option<Bound>), Error> {
	let ty = context.layout_type(layout).unwrap();

	match ty.description() {
		ty::Description::Never => {
			Ok((quote!(::treeldr_rust_prelude::rdf::iter::Empty<V>), None))
		}
		ty::Description::Alias(_, target) => {
			triples_and_values_iterator_of(context, scope, *target, lifetime)
		}
		ty::Description::Primitive(p) => {
			let p_ty = p.generate_with(context, scope).into_tokens()?;
			Ok((
				quote!(::treeldr_rust_prelude::rdf::iter::ValuesOnly<::treeldr_rust_prelude::rdf::iter::LiteralValue<'a, #p_ty, I, V>>),
				Some(Bound::AsLiteral(*p))
			))
		}
		ty::Description::BuiltIn(b) => {
			match b {
				ty::BuiltIn::BTreeSet(item_layout) => {
					let item_ty = context.layout_type(*item_layout).unwrap();
					let mut path = context.module_path(scope).to(&context.parent_module_path(item_ty.module()).unwrap());
					path.push(item_ty.ident());
					Ok((
						quote!(::treeldr_rust_prelude::rdf::iter::Flatten<::std::collection::btree_set::Iter<#lifetime, #path>>),
						None
					))
				}
				ty::BuiltIn::OneOrMany(_) => {
					todo!()
				}
				ty::BuiltIn::Option(item_layout) => {
					let (inner, bound) = triples_and_values_iterator_of(context, scope, *item_layout, lifetime)?;
					Ok((quote!(::treeldr_rust_prelude::rdf::iter::Optional<#inner>), bound))
				}
				ty::BuiltIn::Required(item_layout) => {
					triples_and_values_iterator_of(context, scope, *item_layout, lifetime)
				}
				ty::BuiltIn::Vec(item_layout) => {
					let item_ty = context.layout_type(*item_layout).unwrap();
					let mut path = context.module_path(scope).to(&context.parent_module_path(item_ty.module()).unwrap());
					path.push(item_ty.ident());
					Ok((quote!(::treeldr_rust_prelude::rdf::iter::Flatten<::std::slice::Iter<#lifetime, #path>>), None))
				}
			}
		}
		ty::Description::Struct(s) => {
			let mut path = context.module_path(scope).to(&context.parent_module_path(ty.module()).unwrap());
			path.push(triples_and_values_iterator_name_from(s.ident()));
			Ok((quote!(#path), Some(Bound::TriplesAndValues(layout))))
		}
		ty::Description::Enum(e) => {
			let mut path = context.module_path(scope).to(&context.parent_module_path(ty.module()).unwrap());
			path.push(triples_and_values_iterator_name_from(e.ident()));
			Ok((quote!(#path), Some(Bound::TriplesAndValues(layout))))
		}
		ty::Description::Reference => {
			todo!("reference iterator")
		}
	}
}