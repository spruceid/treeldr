use std::collections::BTreeSet;

use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, Id, IriIndex};

use crate::{
	syntax,
	ty::{structure::Struct, BuiltIn, Description},
	Context, Error, GenerateSyntax,
};

use super::{collect_bounds, from_objects, FromRdfImpl};

impl<'a, M> GenerateSyntax<M> for FromRdfImpl<'a, Struct> {
	type Output = syntax::tr_impl::rdf::FromRdfImpl;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut scope = scope.clone();
		scope.params.identifier = Some(syn::parse2(quote!(N::Id)).unwrap());

		let mut fields_init = Vec::with_capacity(self.ty.fields().len());
		let mut bound_set = BTreeSet::new();

		for field in self.ty.fields() {
			let id = field.ident();

			let init = match field.property() {
				Some(prop_ref) => {
					let prop = context.model().get(prop_ref).unwrap();

					let field_layout_ref = field.layout();
					let field_layout = context.model().get(field_layout_ref).unwrap();
					collect_bounds(context, field_layout_ref, |b| {
						bound_set.insert(b);
					});

					if prop.id()
						== treeldr::Id::Iri(IriIndex::Iri(treeldr::vocab::Term::TreeLdr(
							treeldr::vocab::TreeLdr::Self_,
						))) {
						match field_layout.as_layout().description() {
							treeldr::layout::Description::Required(_) => {
								quote! {
									::treeldr_rust_prelude::Id(id.clone())
								}
							}
							treeldr::layout::Description::Option(_) => {
								quote! {
									Some(::treeldr_rust_prelude::Id(id.clone()))
								}
							}
							_ => {
								quote! {
									Some(::treeldr_rust_prelude::Id(id.clone())).into_iter().collect()
								}
							}
						}
					} else {
						let prop_id = match prop_ref.id() {
							Id::Iri(i) => {
								let iri = context.vocabulary().iri(&i).unwrap().into_str();
								quote! {
									::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
										namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
											#iri
										))
									)
								}
							}
							Id::Blank(_) => return Err(Error::BlankProperty(prop_ref)),
						};

						let objects = quote! { graph.objects(&id, &#prop_id) };

						match field_layout.as_layout().description() {
							treeldr::layout::Description::Required(_) => {
								quote! {
									let mut objects = #objects;

									match objects.next() {
										Some(object) => {
											if objects.next().is_some() {
												panic!("multiples values on functional property")
											}

											::treeldr_rust_prelude::FromRdf::from_rdf(namespace, object, graph)?
										}
										None => {
											return Err(::treeldr_rust_prelude::FromRdfError::MissingRequiredPropertyValue)
										}
									}
								}
							}
							treeldr::layout::Description::Option(_) => {
								match field.ty(context).description() {
									Description::BuiltIn(BuiltIn::Option(_)) => {
										quote! {
											let mut objects = #objects;
											let object = objects.next();
											if objects.next().is_some() {
												panic!("multiples values on functional property")
											}

											match object {
												Some(object) => Some({
													::treeldr_rust_prelude::FromRdf::from_rdf(namespace, object, graph)?
												}),
												None => None
											}
										}
									}
									_ => panic!("not an option"),
								}
							}
							_ => from_objects(field.ty(context), objects),
						}
					}
				}
				None => {
					if field.ty(context).impl_default(context) {
						quote! { ::core::default::Default::default() }
					} else {
						return Err(Error::MissingDefaultImpl);
					}
				}
			};

			fields_init.push(quote! { #id: { #init } })
		}

		let mut bounds = Vec::with_capacity(bound_set.len());
		for b in bound_set {
			bounds.push(b.generate_syntax(context, &scope)?)
		}

		Ok(syntax::tr_impl::rdf::FromRdfImpl {
			type_path: self.ty_ref.generate_syntax(context, &scope)?,
			bounds,
			from_id: quote! {
				Ok(Self {
					#(#fields_init),*
				})
			},
			from_literal: quote!(Err(
				::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue
			)),
		})
	}
}
