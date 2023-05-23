use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use rdf_types::Vocabulary;
use treeldr::{vocab::Primitive, BlankIdIndex, IriIndex, TId};

use crate::{ty, Error, GenerateSyntax};

mod r#enum;
mod r#struct;

/// `RdfQuads` trait implementation.
pub struct RdfQuadsImpl<'a, T> {
	pub ty_ref: TId<treeldr::Layout>,
	pub ty: &'a T,
}

impl<'a, T> RdfQuadsImpl<'a, T> {
	pub fn new(ty_ref: TId<treeldr::Layout>, ty: &'a T) -> Self {
		Self { ty_ref, ty }
	}
}

/// Bound that may appear in a `RdfQuads` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bound {
	/// The given primitive must implement `AsLiteral`.
	AsLiteral(Primitive),
}

impl<M> GenerateSyntax<M> for Bound {
	type Output = syn::WherePredicate;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &crate::Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		match self {
			Self::AsLiteral(p) => {
				let ty = p.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote!(#ty: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>)).unwrap())
			}
		}
	}
}

/// Returns the name of the quads and values iterator derived from the given
/// layout name.
fn quads_and_values_iterator_name_from(ident: &Ident) -> Ident {
	format_ident!("{ident}QuadsAndValues")
}

/// Returns a path to the quads and values iterator of the given layout.
fn quads_and_values_iterator_of<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
	context: &crate::Context<V, M>,
	scope: &crate::Scope,
	layout: TId<treeldr::Layout>,
	lifetime: TokenStream,
) -> Result<syn::Type, Error> {
	let ty = context.layout_type(layout).unwrap();

	match ty.description() {
		ty::Description::Never => {
			Ok(syn::parse2(quote!(::treeldr_rust_prelude::rdf::iter::Empty<V>)).unwrap())
		}
		ty::Description::Alias(a) => {
			quads_and_values_iterator_of(context, scope, a.target(), lifetime)
		}
		ty::Description::Primitive(p) => {
			let p_ty = p.generate_syntax(context, scope)?;
			Ok(
				syn::parse2(quote!(::treeldr_rust_prelude::rdf::ValuesOnly<::treeldr_rust_prelude::rdf::LiteralValue<'r, #p_ty, I, V>>)).unwrap(),
			)
		}
		ty::Description::RestrictedPrimitive(_) => {
			let p_ty = layout.generate_syntax(context, scope)?;
			Ok(
				syn::parse2(quote!(::treeldr_rust_prelude::rdf::ValuesOnly<::treeldr_rust_prelude::rdf::LiteralValue<'r, #p_ty, I, V>>)).unwrap(),
			)
		}
		ty::Description::BuiltIn(b) => match b {
			ty::BuiltIn::BTreeSet(item_layout) => {
				let item_ty_expr = item_layout.generate_syntax(context, scope)?;
				let inner =
					quads_and_values_iterator_of(context, scope, *item_layout, lifetime.clone())?;
				Ok(
					syn::parse2(quote!(::treeldr_rust_prelude::rdf::FlattenQuadsAndValues<
					::std::collections::btree_set::Iter<#lifetime, #item_ty_expr>,
					#inner,
					V
				>))
					.unwrap(),
				)
			}
			ty::BuiltIn::BTreeMap(_, _) => {
				todo!("btreemap triples iterator generator")
			}
			ty::BuiltIn::OneOrMany(_) => {
				todo!("one or many triples iterator generator")
			}
			ty::BuiltIn::Option(item_layout) => {
				let inner = quads_and_values_iterator_of(context, scope, *item_layout, lifetime)?;
				Ok(
					syn::parse2(quote!(::treeldr_rust_prelude::rdf::iter::Optional<#inner>))
						.unwrap(),
				)
			}
			ty::BuiltIn::Required(item_layout) => {
				quads_and_values_iterator_of(context, scope, *item_layout, lifetime)
			}
			ty::BuiltIn::Vec(item_layout) => {
				let item_ty_expr = item_layout.generate_syntax(context, scope)?;
				Ok(syn::parse2(
					quote!(::treeldr_rust_prelude::rdf::iter::Flatten<::std::slice::Iter<#lifetime, #item_ty_expr>>),
				)
				.unwrap())
			}
		},
		ty::Description::Struct(s) => {
			let mut path = context
				.module_path(scope.module)
				.to(&context.parent_module_path(ty.module()).unwrap());
			path.push(quads_and_values_iterator_name_from(s.ident()));
			let path = path.generate_syntax(context, scope)?;
			Ok(syn::parse2(quote!(#path<#lifetime, I, V>)).unwrap())
		}
		ty::Description::Enum(e) => {
			let mut path = context
				.module_path(scope.module)
				.to(&context.parent_module_path(ty.module()).unwrap());
			path.push(quads_and_values_iterator_name_from(e.ident()));
			let path = path.generate_syntax(context, scope)?;
			Ok(syn::parse2(quote!(#path<#lifetime, I, V>)).unwrap())
		}
		ty::Description::Reference(_) => Ok(syn::parse2(quote!(
			::treeldr_rust_prelude::rdf::ValuesOnly<::treeldr_rust_prelude::rdf::IdValue<'r, I, V>>
		))
		.unwrap()),
	}
}

/// Collect the bounds necessary for the `Quads` or `QuadsAndValues`
/// implementations.
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
				ty::Description::Primitive(p) => bound(Bound::AsLiteral(*p)),
				ty::Description::RestrictedPrimitive(r) => stack.push(r.base()),
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
