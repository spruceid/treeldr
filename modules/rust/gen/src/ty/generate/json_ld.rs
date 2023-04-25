//! JSON-LD × Rust code generation.
use crate::{
	ty::{Enum, ParametersValues, Struct},
	Context, Error,
};
use proc_macro2::TokenStream;
use quote::quote;
use treeldr::Id;

use super::GenerateFor;

/// `IntoJsonLd` trait implementation.
pub struct IntoJsonLdImpl;

impl<M> GenerateFor<Struct, M> for IntoJsonLdImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		ty: &Struct,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let mut insert_field = Vec::new();

		for field in ty.fields() {
			if let Some(prop_id) = field.property() {
				let rust_prop_id = match prop_id.into_id() {
					Id::Iri(iri) => {
						let iri = context.vocabulary().iri(&iri).unwrap().to_string();
						quote!(::treeldr_rust_prelude::json_ld::ValidId::Iri(
							vocabulary.insert(::treeldr_rust_prelude::static_iref::iri!(#iri))
						))
					}
					Id::Blank(blank) => {
						let blank = context.vocabulary().blank_id(&blank).unwrap().to_string();
						quote!(::treeldr_rust_prelude::json_ld::ValidId::Blank(
							vocabulary.insert_blank_id(::treeldr_rust_prelude::rdf_types::BlankId::new(#blank).unwrap())
						))
					}
				};

				// let key = field.name().as_str();
				let id = field.ident();

				let layout_ref = field.layout();
				let layout = context.model().get(layout_ref).unwrap();

				insert_field.push(match layout.as_layout().description() {
					treeldr::layout::Description::Required(_) => {
						quote! {
							result.properties_mut().insert(
								::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Id::Valid(#rust_prop_id), ()),
								::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(self.#id, vocabulary, meta)
							);
						}
					}
					treeldr::layout::Description::Option(_) => {
						quote! {
							if let Some(value) = self.#id {
								result.properties_mut().insert(
									::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Id::Valid(#rust_prop_id), ()),
									::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(value, vocabulary, meta)
								);
							}
						}
					}
					_ => {
						quote! {
							if !self.#id.is_empty() {
								let list = ::treeldr_rust_prelude::json_ld::object::List::new(
									(),
									::treeldr_rust_prelude::locspan::Meta(
										self.#id.into_iter()
											.map(|v| ::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(v, vocabulary, meta)).collect(),
										()
									)
								);

								result.properties_mut().insert(
									::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Id::Valid(#rust_prop_id), ()),
									::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::Indexed::new(::treeldr_rust_prelude::json_ld::Object::List(list), None), ())
								);
							}
						}
					}
				})
			}
		}

		let ident = ty.ident();
		let params_values = ParametersValues::new_for_type(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);
		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::VocabularyMut> ::treeldr_rust_prelude::IntoJsonLdObjectMeta<N> for #ident #params
			where
				N: treeldr_rust_prelude::rdf_types::Namespace,
				N::Id: ::treeldr_rust_prelude::rdf_types::IntoId<Iri = N::Iri, BlankId = N::BlankId>,
				N::Iri: ::core::cmp::Eq + ::std::hash::Hash,
				N::BlankId: ::core::cmp::Eq + ::std::hash::Hash
			{
				fn into_json_ld_object_meta(
					self,
					vocabulary: &mut N,
					meta: ()
				) -> ::treeldr_rust_prelude::json_ld::IndexedObject<N::Iri, N::BlankId, ()> {
					let mut result = ::treeldr_rust_prelude::json_ld::Node::new();
					#(#insert_field)*
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::Indexed::new(::treeldr_rust_prelude::json_ld::Object::Node(Box::new(result)), None),
						meta
					)
				}
			}
		});

		Ok(())
	}
}

impl<M> GenerateFor<Enum, M> for IntoJsonLdImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ident = ty.ident();
		let params_values = ParametersValues::new_for_type(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);

		let variants = ty.variants().iter().map(|variant| {
			let v_ident = variant.ident();
			if variant.ty().is_some() {
				quote! {
					Self::#v_ident(value) => {
						value.into_json_ld_object_meta(vocabulary, meta)
					}
				}
			} else {
				quote! {
					Self::#v_ident => {
						::treeldr_rust_prelude::locspan::Meta(
							::treeldr_rust_prelude::json_ld::Indexed::new(
								::treeldr_rust_prelude::json_ld::Object::Node(Box::new(
									::treeldr_rust_prelude::json_ld::Node::new()
								)),
								None
							),
							meta
						)
					}
				}
			}
		});

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::VocabularyMut> ::treeldr_rust_prelude::IntoJsonLdObjectMeta<N> for #ident #params
			where
				N: treeldr_rust_prelude::rdf_types::Namespace,
				N::Id: ::treeldr_rust_prelude::rdf_types::IntoId<Iri = N::Iri, BlankId = N::BlankId>,
				N::Iri: ::core::cmp::Eq + ::std::hash::Hash,
				N::BlankId: ::core::cmp::Eq + ::std::hash::Hash
			{
				fn into_json_ld_object_meta(
					self,
					vocabulary: &mut N,
					meta: ()
				) -> ::treeldr_rust_prelude::json_ld::IndexedObject<N::Iri, N::BlankId, ()> {
					match self {
						#(#variants,)*
					}
				}
			}
		});

		Ok(())
	}
}

pub struct IntoJsonLdSyntaxImpl;

impl<M> GenerateFor<Struct, M> for IntoJsonLdSyntaxImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		ty: &Struct,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let mut insert_field = Vec::new();

		for field in ty.fields() {
			let key = field.name().as_str();
			let id = field.ident();

			let layout_ref = field.layout();
			let layout = context.model().get(layout_ref).unwrap();

			insert_field.push(match layout.as_layout().description() {
				treeldr::layout::Description::Required(_) => {
					quote! {
						result.insert(
							::treeldr_rust_prelude::locspan::Meta(#key.into(), ()),
							::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(self.#id, namespace), ())
						);
					}
				}
				treeldr::layout::Description::Option(_) => {
					quote! {
						if let Some(value) = self.#id {
							result.insert(
								::treeldr_rust_prelude::locspan::Meta(#key.into(), ()),
								::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(value, namespace), ())
							);
						}
					}
				}
				_ => {
					quote! {
						if !self.#id.is_empty() {
							result.insert(
								::treeldr_rust_prelude::locspan::Meta(#key.into(), ()),
								::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::json_ld::syntax::Value::Array(
									self.#id.into_iter()
										.map(|v| ::locspan::Meta(::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(v, namespace), ())).collect()
								), ())
							);
						}
					}
				}
			})
		}

		let ident = ty.ident();
		let params_values = ParametersValues::new_for_type(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);
		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLdSyntax<N> for #ident #params where N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N> {
				fn into_json_ld_syntax(
					self,
					namespace: &N
				) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
					let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
					#(#insert_field)*
					result.into()
				}
			}
		});

		Ok(())
	}
}

impl<M> GenerateFor<Enum, M> for IntoJsonLdSyntaxImpl {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ident = ty.ident();
		let params_values = ParametersValues::new_for_type(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);

		let variants = ty.variants().iter().map(|variant| {
			let v_ident = variant.ident();
			if variant.ty().is_some() {
				quote! {
					Self::#v_ident(value) => {
						value.into_json_ld_syntax(namespace)
					}
				}
			} else {
				quote! {
					Self::#v_ident => {
						::treeldr_rust_prelude::json_ld::syntax::Value::Null
					}
				}
			}
		});

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLdSyntax<N> for #ident #params where N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N> {
				fn into_json_ld_syntax(
					self,
					namespace: &N
				) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
					match self {
						#(#variants,)*
					}
				}
			}
		});

		Ok(())
	}
}
