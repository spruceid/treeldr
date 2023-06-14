use super::FromRdfError;
use iref::Iri;
use rdf_types::{IriVocabularyMut, Literal};

mod xsd;

/// Convert from an RDF literal value.
pub trait FromLiteral<L, N>: Sized {
	fn from_literal_type_unchecked(literal: &L) -> Result<Self, FromRdfError>;

	fn from_literal(namespace: &N, literal: &L) -> Result<Self, FromRdfError>;
}

fn type_check<'l, V: IriVocabularyMut<Iri = T>, S, T, L>(
	vocabulary: &V,
	literal: &'l Literal<rdf_types::literal::Type<T, L>, S>,
	expected_ty: Iri,
) -> Result<&'l S, FromRdfError> {
	match literal.type_() {
		rdf_types::literal::Type::Any(ty) => {
			if vocabulary.iri(ty).unwrap() == expected_ty {
				Ok(literal.value())
			} else {
				Err(FromRdfError::UnexpectedType)
			}
		}
		_ => Err(FromRdfError::UnexpectedType),
	}
}
