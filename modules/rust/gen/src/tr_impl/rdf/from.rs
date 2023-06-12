use std::collections::HashSet;

use crate::{
	ty::{self, BuiltIn, Description, Type},
	Error, GenerateSyntax,
};
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

mod r#enum;
mod primitive;
mod r#struct;

/// `FromRdf` trait implementation.
pub struct FromRdfImpl<'a, T> {
	ty_ref: TId<treeldr::Layout>,
	ty: &'a T,
}

impl<'a, T> FromRdfImpl<'a, T> {
	pub fn new(ty_ref: TId<treeldr::Layout>, ty: &'a T) -> Self {
		Self { ty_ref, ty }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bound {
	FromLiteral(TId<treeldr::Layout>),
}

impl<M> GenerateSyntax<M> for Bound {
	type Output = syn::WherePredicate;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &crate::Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		match self {
			Self::FromLiteral(p) => {
				let ty = p.generate_syntax(context, scope)?;
				Ok(
					syn::parse2(quote!(#ty: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>))
						.unwrap(),
				)
			}
		}
	}
}

/// Collect the bounds necessary for the `FromRdf` implementation of the given
/// layout.
fn collect_bounds<V, M>(
	context: &crate::Context<V, M>,
	layout: TId<treeldr::Layout>,
	mut bound: impl FnMut(Bound),
) {
	let mut stack = vec![layout];
	let mut visited = HashSet::new();

	while let Some(layout) = stack.pop() {
		if visited.insert(layout) {
			let ty = context.layout_type(layout).unwrap();

			match ty.description() {
				ty::Description::Never => (),
				ty::Description::Alias(a) => stack.push(a.target()),
				ty::Description::Primitive(_) => bound(Bound::FromLiteral(layout)),
				ty::Description::DerivedPrimitive(_) => bound(Bound::FromLiteral(layout)),
				ty::Description::BuiltIn(b) => match b {
					ty::BuiltIn::BTreeMap(key_layout, value_layout) => {
						stack.push(*key_layout);
						stack.push(*value_layout)
					}
					ty::BuiltIn::BTreeSet(item_layout)
					| ty::BuiltIn::OneOrMany(item_layout)
					| ty::BuiltIn::Option(item_layout)
					| ty::BuiltIn::Required(item_layout)
					| ty::BuiltIn::Vec(item_layout) => stack.push(*item_layout),
				},
				ty::Description::Struct(s) => {
					for field in s.fields() {
						stack.push(field.layout())
					}
				}
				ty::Description::Enum(e) => {
					for variant in e.variants() {
						if let Some(layout_ref) = variant.ty() {
							stack.push(layout_ref)
						}
					}
				}
				ty::Description::Reference(_) => (),
			}
		}
	}
}

fn from_objects(ty: &Type, objects: TokenStream) -> TokenStream {
	match ty.description() {
		Description::BuiltIn(BuiltIn::Vec(_item)) => {
			quote! {
				let mut result = ::std::vec::Vec::new();
				for object in #objects {
					result.push(::treeldr_rust_prelude::FromRdf::from_rdf(namespace, object, graph)?)
				}
				result
			}
		}
		Description::BuiltIn(BuiltIn::BTreeSet(_item)) => {
			quote! {
				let mut result = ::std::collections::btree_set::BTreeSet::new();
				for object in #objects {
					result.insert(::treeldr_rust_prelude::FromRdf::from_rdf(namespace, object, graph)?);
				}
				result
			}
		}
		Description::Alias(_) => {
			quote! {
				todo!("alias from RDF")
			}
		}
		_ => panic!("not a collection type"),
	}
}
