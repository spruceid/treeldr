use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, Id, IriIndex};

use crate::{
	ty::{
		generate::GenerateFor, params::ParametersValues, structure::Struct, BuiltIn, Description,
	},
	Context, Error, Generate, GenerateList,
};

use super::{collect_bounds, from_objects, FromRdfImpl};

impl<M> GenerateFor<Struct, M> for FromRdfImpl {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Struct,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let mut fields_init = Vec::with_capacity(ty.fields().len());
		let mut bounds = BTreeSet::new();

		for field in ty.fields() {
			let id = field.ident();

			let init = match field.property() {
				Some(prop_ref) => {
					let prop = context.model().get(prop_ref).unwrap();

					let field_layout_ref = field.layout();
					let field_layout = context.model().get(field_layout_ref).unwrap();
					collect_bounds(context, field_layout_ref, |b| {
						bounds.insert(b);
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

		let ident = ty.ident();
		let params_values = ParametersValues::new_for_type(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);

		let bounds = bounds
			.separated_by(&quote!(,))
			.generate_with(context, scope)
			.into_tokens()?;

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V> for #ident #params
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri=N::Iri>,
				V: ::treeldr_rust_prelude::rdf::TypeCheck<N::Id>,
				#bounds
			{
				fn from_rdf<G>(
					namespace: &mut N,
					value: &::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
					graph: &G
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
				where
					G: ::treeldr_rust_prelude::grdf::Graph<
						Subject=N::Id,
						Predicate=N::Id,
						Object=::treeldr_rust_prelude::rdf_types::Object<N::Id, V>
					>
				{
					match value {
						::treeldr_rust_prelude::rdf_types::Object::Id(id) => {
							Ok(Self {
								#(#fields_init),*
							})
						}
						::treeldr_rust_prelude::rdf_types::Object::Literal(_) => {
							Err(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue)
						}
					}
				}
			}
		});

		Ok(())
	}
}
