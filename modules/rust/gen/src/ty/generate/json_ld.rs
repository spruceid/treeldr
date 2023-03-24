//! JSON-LD × Rust code generation.
use crate::{
	ty::{Enum, ParametersValues, Struct},
	Context, Error,
};
use proc_macro2::TokenStream;
use quote::quote;

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
			let key = field.name().as_str();
			let id = field.ident();

			let layout_ref = field.layout();
			let layout = context.model().get(layout_ref).unwrap();

			insert_field.push(match layout.as_layout().description() {
				treeldr::layout::Description::Required(_) => {
					quote! {
						result.insert(
							::treeldr_rust_prelude::locspan::Meta(#key.into(), ()),
							::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::IntoJsonLd::into_json_ld(self.#id, namespace), ())
						);
					}
				}
				treeldr::layout::Description::Option(_) => {
					quote! {
						if let Some(value) = self.#id {
							result.insert(
								::treeldr_rust_prelude::locspan::Meta(#key.into(), ()),
								::treeldr_rust_prelude::locspan::Meta(::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace), ())
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
										.map(|v| ::locspan::Meta(::treeldr_rust_prelude::IntoJsonLd::into_json_ld(v, namespace), ())).collect()
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
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N> for #ident #params where N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N> {
				fn into_json_ld(
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
						value.into_json_ld(namespace)
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
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N> for #ident #params where N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N> {
				fn into_json_ld(
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
