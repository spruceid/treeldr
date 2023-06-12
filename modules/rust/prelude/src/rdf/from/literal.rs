use super::FromRdfError;
use iref::Iri;
use rdf_types::{IriVocabularyMut, Literal};
use static_iref::iri;

mod xsd;

/// Convert from an RDF literal value.
pub trait FromLiteral<L, N>: Sized {
	fn from_literal_type_unchecked(literal: &L) -> Result<Self, FromRdfError>;

	fn from_literal(namespace: &N, literal: &L) -> Result<Self, FromRdfError>;
}

fn type_check<'l, V: IriVocabularyMut<Iri = T>, S, T, L>(
	vocabulary: &V,
	literal: &'l Literal<S, T, L>,
	expected_ty: Iri,
) -> Result<&'l S, FromRdfError> {
	match literal {
		Literal::TypedString(s, ty) => {
			if vocabulary.iri(ty).unwrap() == expected_ty {
				Ok(s)
			} else {
				Err(FromRdfError::UnexpectedType)
			}
		}
		Literal::String(s) => {
			if expected_ty == iri!("http://www.w3.org/2001/XMLSchema#string") {
				Ok(s)
			} else {
				Err(FromRdfError::UnexpectedType)
			}
		}
		Literal::LangString(_, _) => Err(FromRdfError::UnexpectedType),
	}
}
