use std::collections::BTreeSet;

use proc_macro2::Ident;
use quote::quote;
use treeldr::Id;

use crate::{
	ty::{self, structure::Struct},
	GenerateSyntax, syntax
};

use super::{
	collect_bounds, quads_and_values_iterator_name_from, quads_and_values_iterator_of, RdfQuadsImpl,
};

impl<'a, M> GenerateSyntax<M> for RdfQuadsImpl<'a, Struct> {
	type Output = syntax::tr_impl::rdf::QuadsImpl;

	fn generate_syntax<V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>>(
		&self,
		context: &crate::Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, crate::Error> {
		let ident = self.ty.ident();
		let iterator_ident = quads_and_values_iterator_name_from(ident);

		let mut iterator_fields = Vec::with_capacity(self.ty.fields().len());
		let mut iterator_id_init = None;

		let mut next_body = quote!(self
			.id_
			.take()
			.map(::treeldr_rust_prelude::rdf_types::Object::Id)
			.map(::treeldr_rust_prelude::rdf::QuadOrValue::Value));

		let mut bounds_set = BTreeSet::new();
		for field in self.ty.fields() {
			let field_ident = field.ident();
			if field.is_self() {
				let ty = context.layout_type(field.layout()).unwrap();
				iterator_id_init = Some(match ty.description() {
					ty::Description::BuiltIn(ty::BuiltIn::Option(_)) => {
						quote! {
							self.#field_ident.clone().map(::treeldr_rust_prelude::Id::unwrap).unwrap_or_else(|| {
								generator.next(namespace)
							})
						}
					}
					ty::Description::BuiltIn(ty::BuiltIn::Required(_)) => {
						quote! {
							self.#field_ident.clone().unwrap()
						}
					}
					_ => panic!("invalid `tldr:self` layout"),
				})
			} else {
				let iter_ty = quads_and_values_iterator_of(
					context,
					scope,
					field.layout(),
					quote!('a),
				)?;

				collect_bounds(context, field.layout(), |b| {
					bounds_set.insert(b);
				});

				iterator_fields.push(syntax::tr_impl::rdf::IteratorField {
					ident: field_ident.clone(),
					ty: iter_ty,
					init: quote! {
						#field_ident: self.#field_ident.unbound_rdf_quads_and_values(namespace, generator)
					}
				});

				let mut prop_iri = None;
				if let Some(prop_id) = field.property() {
					if let Id::Iri(iri_index) = prop_id.id() {
						prop_iri = Some(iri_index);
					}
				}

				let map_prop_item = match prop_iri {
					Some(iri_index) => {
						let prop_iri = context.vocabulary().iri(&iri_index).unwrap().into_str();
						quote! {
							.map(|item| match item {
								::treeldr_rust_prelude::rdf::QuadOrValue::Quad(quad) => {
									treeldr_rust_prelude::rdf::QuadOrValue::Quad(quad)
								}
								treeldr_rust_prelude::rdf::QuadOrValue::Value(value) => {
									treeldr_rust_prelude::rdf::QuadOrValue::Quad(::rdf_types::Quad(
										self.id_.clone().unwrap(),
										treeldr_rust_prelude::rdf_types::FromIri::from_iri(
											vocabulary.insert(::treeldr_rust_prelude::static_iref::iri!(#prop_iri))
										),
										value,
										graph.cloned()
									))
								}
							})
						}
					}
					None => {
						quote! {
							.filer_map(|item| match item {
								treeldr_rust_prelude::rdf::QuadOrValue::Quad(quad) => Some(treeldr_rust_prelude::rdf::QuadOrValue::Quad(quad)),
								treeldr_rust_prelude::rdf::QuadOrValue::Value(value) => None
							})
						}
					}
				};

				next_body = quote! {
					self.#field_ident
						.next_with(
							vocabulary,
							generator,
							graph
						)
						#map_prop_item
						.or_else(|| #next_body)
				};
			}
		}

		if iterator_fields.is_empty() {
			iterator_fields.push(syntax::tr_impl::rdf::IteratorField {
				ident: Ident::new("_v", proc_macro2::Span::call_site()),
				ty: syn::parse2(quote!(_v: ::std::marker::PhantomData<&'a V>)).unwrap(),
				init: quote!(::std::marker::PhantomData)
			})
		}

		let iterator_id_init = iterator_id_init.unwrap_or_else(|| {
			quote! {
				generator.next(namespace)
			}
		});

		let mut bounds = Vec::with_capacity(bounds_set.len());
		for b in bounds_set {
			bounds.push(b.generate_syntax(context, scope)?)
		}

		let type_path = self.ty_ref.generate_syntax(context, scope)?;

		Ok(syntax::tr_impl::rdf::QuadsImpl {
			type_path,
			iterator_ty: syntax::tr_impl::rdf::IteratorType::Struct(
				syntax::tr_impl::rdf::IteratorStruct {
					ident: iterator_ident,
					fields: iterator_fields,
					id_init: iterator_id_init,
					next_body
				}
			),
			bounds
		})
	}
}
