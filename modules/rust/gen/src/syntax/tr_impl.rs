use quote::ToTokens;

pub mod class;
pub mod json_ld;
pub mod rdf;

pub enum TraitImplementation {
	ClassTrait(class::TraitImpl),
	RdfQuads(rdf::QuadsImpl),
	FromRdf(rdf::FromRdfImpl),
	AsJsonLd(json_ld::AsJsonLdImpl),
	IntoJsonLd(json_ld::IntoJsonLdImpl),
	IntoJsonLdSyntax(json_ld::IntoJsonLdSyntaxImpl),
}

impl ToTokens for TraitImplementation {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::ClassTrait(i) => i.to_tokens(tokens),
			Self::RdfQuads(i) => i.to_tokens(tokens),
			Self::FromRdf(i) => i.to_tokens(tokens),
			Self::AsJsonLd(i) => i.to_tokens(tokens),
			Self::IntoJsonLd(i) => i.to_tokens(tokens),
			Self::IntoJsonLdSyntax(i) => i.to_tokens(tokens),
		}
	}
}
