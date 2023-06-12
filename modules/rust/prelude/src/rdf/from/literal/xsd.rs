use rdf_types::{IriVocabularyMut, Literal};
use static_iref::iri;
use xsd_types::ParseRdf;

use crate::FromRdfError;

use super::{type_check, FromLiteral};

macro_rules! impl_from_literal {
	{ $($ty:ty : $rdf_ty:tt),* } => {
		$(
			impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V> for $ty {
				fn from_literal_type_unchecked(literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
					match <$ty>::parse_rdf(literal.string_literal().as_ref()) {
						Ok(i) => Ok(i),
						Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
					}
				}

				fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
					type_check(
						vocabulary,
						literal,
						iri!($rdf_ty),
					)?;

					FromLiteral::<Literal<S, T, L>, V>::from_literal_type_unchecked(literal)
				}
			}
		)*
	};
}

impl_from_literal! {
	xsd_types::Boolean: "http://www.w3.org/2001/XMLSchema#boolean",
	xsd_types::Decimal: "http://www.w3.org/2001/XMLSchema#decimal",
	xsd_types::Integer: "http://www.w3.org/2001/XMLSchema#integer",
	xsd_types::Long: "http://www.w3.org/2001/XMLSchema#long",
	xsd_types::Int: "http://www.w3.org/2001/XMLSchema#int",
	xsd_types::Short: "http://www.w3.org/2001/XMLSchema#short",
	xsd_types::Byte: "http://www.w3.org/2001/XMLSchema#byte",
	xsd_types::NonNegativeInteger: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
	xsd_types::PositiveInteger: "http://www.w3.org/2001/XMLSchema#positiveInteger",
	xsd_types::UnsignedLong: "http://www.w3.org/2001/XMLSchema#unsignedLong",
	xsd_types::UnsignedInt: "http://www.w3.org/2001/XMLSchema#unsignedInt",
	xsd_types::UnsignedShort: "http://www.w3.org/2001/XMLSchema#unsignedShort",
	xsd_types::UnsignedByte: "http://www.w3.org/2001/XMLSchema#unsignedByte",
	xsd_types::NonPositiveInteger: "http://www.w3.org/2001/XMLSchema#nonPositiveInteger",
	xsd_types::NegativeInteger: "http://www.w3.org/2001/XMLSchema#negativeInteger",
	xsd_types::Double: "http://www.w3.org/2001/XMLSchema#double",
	xsd_types::Float: "http://www.w3.org/2001/XMLSchema#float",
	xsd_types::String: "http://www.w3.org/2001/XMLSchema#string",
	xsd_types::Base64BinaryBuf: "http://www.w3.org/2001/XMLSchema#base64Binary",
	xsd_types::HexBinaryBuf: "http://www.w3.org/2001/XMLSchema#hexBinary"
}

impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V>
	for ::chrono::NaiveDate
{
	fn from_literal_type_unchecked(literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		match literal.string_literal().as_ref().parse::<Self>() {
			Ok(d) => Ok(d),
			Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}

	fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		type_check(
			vocabulary,
			literal,
			iri!("http://www.w3.org/2001/XMLSchema#date"),
		)?;

		FromLiteral::<Literal<S, T, L>, V>::from_literal_type_unchecked(literal)
	}
}

impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V>
	for ::chrono::DateTime<::chrono::Utc>
{
	fn from_literal_type_unchecked(literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		match literal.string_literal().as_ref().parse::<Self>() {
			Ok(d) => Ok(d),
			Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}

	fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		type_check(
			vocabulary,
			literal,
			iri!("http://www.w3.org/2001/XMLSchema#dateTime"),
		)?;

		FromLiteral::<Literal<S, T, L>, V>::from_literal_type_unchecked(literal)
	}
}

impl<S: AsRef<str>, T, L, V: IriVocabularyMut<Iri = T>> FromLiteral<Literal<S, T, L>, V>
	for iref::IriBuf
{
	fn from_literal_type_unchecked(literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		match literal.string_literal().as_ref().parse::<Self>() {
			Ok(d) => Ok(d),
			Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}

	fn from_literal(vocabulary: &V, literal: &Literal<S, T, L>) -> Result<Self, FromRdfError> {
		type_check(
			vocabulary,
			literal,
			iri!("http://www.w3.org/2001/XMLSchema#anyURI"),
		)?;

		FromLiteral::<Literal<S, T, L>, V>::from_literal_type_unchecked(literal)
	}
}
