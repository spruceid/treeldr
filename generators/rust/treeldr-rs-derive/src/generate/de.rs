use proc_macro2::{Span, TokenStream};
use syn::DeriveInput;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	// ...
}

impl Error {
	pub fn span(&self) -> Span {
		todo!()
	}
}

pub fn generate(input: DeriveInput) -> Result<TokenStream, Error> {
	todo!()
}
