//! JSON-LD × Rust code generation.
use crate::{
	ty::{Enum, Struct},
	Context,
};
use proc_macro2::TokenStream;
use quote::quote;

pub enum Error {
	// ...
}

/// Creates a JSON-LD node object from a structure.
pub fn structure_builder<F>(
	context: &Context<F>,
	ty: &Struct<F>,
	ident: &proc_macro2::Ident,
) -> Result<TokenStream, Error> {
	let mut insert_field = Vec::new();

	for field in ty.fields() {
		let key = field.name().as_str();
		let id = field.ident();

		let layout_ref = field.layout();
		let layout = context.model().layouts().get(layout_ref).unwrap();

		insert_field.push(match layout.description() {
			treeldr::layout::Description::Required(_) => {
				quote! {
					result.insert(
						::locspan::Meta(#key.into(), ()),
						::locspan::Meta(::treeldr_rust_prelude::IntoJsonLd::into_json_ld(self.#id), ())
					);
				}
			}
			treeldr::layout::Description::Option(_) => {
				quote! {
					if let Some(value) = self.#id {
						result.insert(
							::locspan::Meta(#key.into(), ()),
							::locspan::Meta(::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value), ())
						);
					}
				}
			}
			_ => {
				quote! {
					if !self.#id.is_empty() {
						result.insert(
							::locspan::Meta(#key.into(), ()),
							::locspan::Meta(::json_ld::syntax::Value::Array(
								self.#id.into_iter()
									.map(|v| ::locspan::Meta(::treeldr_rust_prelude::IntoJsonLd::into_json_ld(v), ())).collect()
							), ())
						);
					}
				}
			}
		})
	}

	Ok(quote! {
		impl ::treeldr_rust_prelude::IntoJsonLd<()> for #ident {
			fn into_json_ld(self) -> ::json_ld::syntax::Value<()> {
				let mut result = json_ld::syntax::Object::new();

				#(#insert_field)*

				result.into()
			}
		}
	})
}

/// Creates a JSON-LD node object from an enumeration.
pub fn enum_builder<F>(
	_context: &Context<F>,
	_ty: &Enum<F>,
	ident: &proc_macro2::Ident,
) -> Result<TokenStream, Error> {
	Ok(quote! {
		impl ::treeldr_rust_prelude::IntoJsonLd<()> for #ident {
			fn into_json_ld(self) -> ::json_ld::syntax::Value<()> {
				todo!()
			}
		}
	})
}
