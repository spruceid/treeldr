//! This library defines the derive macros for the `linked_data` library. It is
//! not meant to be used directly. It is reexported by the `linked_data`
//! library.
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use syn::DeriveInput;

mod parse;
mod generate;

#[proc_macro_derive(Serialize, attributes(tldr))]
#[proc_macro_error]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as DeriveInput);
	let mut output = proc_macro2::TokenStream::new();

	match generate::ser::generate(input) {
		Ok(tokens) => output.extend(tokens),
		Err(e) => {
			abort!(e.span(), e)
		}
	}

	output.into()
}

#[proc_macro_derive(Deserialize, attributes(tldr))]
#[proc_macro_error]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as DeriveInput);
	let mut output = proc_macro2::TokenStream::new();

	match generate::de::generate(input) {
		Ok(tokens) => output.extend(tokens),
		Err(e) => {
			abort!(e.span(), e)
		}
	}

	output.into()
}