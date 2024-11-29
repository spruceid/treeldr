use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use syn::parse_macro_input;

pub(crate) mod attributes;
mod de;
mod ser;

#[proc_macro_derive(SerializeIndexed, attributes(serde_indexed))]
#[proc_macro_error]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as syn::DeriveInput);
	match ser::derive(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => {
			abort!(e.span(), "{e}")
		}
	}
}

#[proc_macro_derive(DeserializeIndexed, attributes(serde_indexed))]
#[proc_macro_error]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as syn::DeriveInput);
	match de::derive(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => {
			abort!(e.span(), "{e}")
		}
	}
}
