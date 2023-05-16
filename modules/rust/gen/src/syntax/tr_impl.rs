use quote::ToTokens;

pub mod class;
pub mod rdf;
pub mod json_ld;

pub enum TraitImplementation {
	ClassTraitImpl(class::TraitImpl)
}

impl ToTokens for TraitImplementation {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::ClassTraitImpl(i) => i.to_tokens(tokens)
		}
	}
}