use iref::Iri;
use rdf_types::{Literal, IriVocabularyMut};
use super::FromRdfError;

mod xsd;

/// Convert from an RDF literal value.
pub trait FromLiteral<L, N>: Sized {
	fn from_literal(
		namespace: &N,
		literal: &L
	) -> Result<Self, FromRdfError>;
}

fn type_check<'l, V: IriVocabularyMut<Iri = T>, S, T, L>(
	vocabulary: &V,
	literal: &'l Literal<S, T, L>,
	expected_ty: Iri
) -> Result<&'l S, FromRdfError> {
	match literal {
		Literal::TypedString(s, ty) => {
			if vocabulary.iri(ty).unwrap() == expected_ty {
				Ok(s)
			} else {
				Err(FromRdfError::UnexpectedType)
			}
		},
		Literal::String(_) => Err(FromRdfError::UnexpectedType),
		Literal::LangString(_, _) => Err(FromRdfError::UnexpectedType),
	}
}