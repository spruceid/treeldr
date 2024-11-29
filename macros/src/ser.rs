use proc_macro2::{Span, TokenStream};
use quote::quote;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("unsupported `union` type")]
	Union(Span),
}

impl Error {
	pub fn span(&self) -> Span {
		match self {
			Self::Union(s) => *s,
		}
	}
}

pub fn derive(input: syn::DeriveInput) -> Result<TokenStream, Error> {
	let ident = &input.ident;

	let has_resource = !input.generics.params.is_empty();

	let resource_arg = has_resource.then_some(quote! { <R> });

	let body = match input.data {
		syn::Data::Struct(s) => {
			quote! { todo!() }
		}
		syn::Data::Enum(e) => {
			quote! { todo!() }
		}
		syn::Data::Union(u) => return Err(Error::Union(u.union_token.span)),
	};

	Ok(quote! {
		impl<R> crate::rdf::serde::SerializeIndexed<R> for #ident #resource_arg {
			fn serialize_indexed<S>(
				&self,
				index: &crate::rdf::ResourceForwardIndex<R>,
				serializer: S
			) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
				todo!()
			}
		}
	})
}
