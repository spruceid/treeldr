use rdf_types::{IriVocabularyMut, Literal};
use static_iref::iri;

use crate::FromRdfError;

use super::{type_check, FromLiteral};

impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V> for bool {
	fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		let lexical = type_check(
			vocabulary,
			literal,
			iri!("http://www.w3.org/2001/XMLSchema#boolean"),
		)?;
		match lexical.as_ref() {
			"true" => Ok(true),
			"false" => Ok(false),
			_ => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}
}

impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V>
	for String
{
	fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		match literal {
			Literal::String(s) => Ok(s.as_ref().to_owned()),
			Literal::TypedString(s, ty) => {
				if vocabulary.iri(ty).unwrap() == iri!("http://www.w3.org/2001/XMLSchema#string") {
					Ok(s.as_ref().to_owned())
				} else {
					Err(FromRdfError::UnexpectedType)
				}
			}
			Literal::LangString(_, _) => Err(FromRdfError::UnexpectedType),
		}
	}
}

impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V> for i64 {
	fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		let lexical = type_check(
			vocabulary,
			literal,
			iri!("http://www.w3.org/2001/XMLSchema#integer"),
		)?;
		match xsd_types::lexical::Integer::new(lexical.as_ref()) {
			Ok(i) => Ok(i.as_str().parse().unwrap()),
			Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}
}

impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V>
	for ::chrono::DateTime<::chrono::Utc>
{
	fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		let lexical = type_check(
			vocabulary,
			literal,
			iri!("http://www.w3.org/2001/XMLSchema#dateTime"),
		)?;
		match lexical.as_ref().parse::<Self>() {
			Ok(d) => Ok(d),
			Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}
}

impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V>
	for iref::IriBuf
{
	fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		let lexical = type_check(
			vocabulary,
			literal,
			iri!("http://www.w3.org/2001/XMLSchema#anyURI"),
		)?;
		match lexical.as_ref().parse::<Self>() {
			Ok(d) => Ok(d),
			Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}
}
