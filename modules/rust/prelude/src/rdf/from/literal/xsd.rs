use rdf_types::{
	Interpretation, IriVocabulary, LanguageTagVocabulary, Literal, LiteralVocabulary,
	ReverseLiteralInterpretation, Vocabulary,
};
use static_iref::iri;
use std::str::FromStr;
use xsd_types::ParseRdf;

use crate::{rdf::TypeCheck, FromRdf, FromRdfError, FromRdfLiteral};

macro_rules! impl_from_literal {
	{ $($ty:ty : $rdf_ty:tt = $method:ident),* } => {
		$(
			impl<V> FromRdfLiteral<V> for $ty
			where
				V: Vocabulary<Type = rdf_types::literal::Type<<V as IriVocabulary>::Iri, <V as LanguageTagVocabulary>::LanguageTag>>,
				V::Value: AsRef<str>
			{
				fn from_rdf_literal_value(
					value: &V::Value
				) -> Result<Self, FromRdfError> {
					match <$ty>::$method(value.as_ref()) {
						Ok(i) => Ok(i),
						Err(_) => Err(FromRdfError::InvalidLexicalRepresentation)
					}
				}

				fn from_rdf_literal(
					vocabulary: &V,
					literal: &Literal<V::Type, V::Value>
				) -> Result<Self, FromRdfError> {
					if literal.has_type(vocabulary, iri!($rdf_ty)) {
						FromRdfLiteral::<V>::from_rdf_literal_value(literal.value())
					} else {
						Err(FromRdfError::UnexpectedType)
					}
				}
			}

			impl<V: LiteralVocabulary, I: ReverseLiteralInterpretation<Literal = V::Literal>> FromRdf<V, I> for $ty
			where
				$ty: FromRdfLiteral<V>
			{
				fn from_rdf<G>(
					vocabulary: &V,
					interpretation: &I,
					_graph: &G,
					id: &<I as Interpretation>::Resource
				) -> Result<Self, FromRdfError>
				where
					G: grdf::Graph<Subject = <I as Interpretation>::Resource, Predicate = <I as Interpretation>::Resource, Object = <I as Interpretation>::Resource>
				{
					let literal_id = interpretation.literals_of(id).next().ok_or(FromRdfError::ExpectedLiteralValue)?;
					let literal = vocabulary.literal(literal_id).unwrap();
					Self::from_rdf_literal(vocabulary, literal)
				}
			}
		)*
	};
}

impl_from_literal! {
	xsd_types::Boolean: "http://www.w3.org/2001/XMLSchema#boolean" = parse_rdf,
	xsd_types::Decimal: "http://www.w3.org/2001/XMLSchema#decimal" = parse_rdf,
	xsd_types::Integer: "http://www.w3.org/2001/XMLSchema#integer" = parse_rdf,
	xsd_types::Long: "http://www.w3.org/2001/XMLSchema#long" = parse_rdf,
	xsd_types::Int: "http://www.w3.org/2001/XMLSchema#int" = parse_rdf,
	xsd_types::Short: "http://www.w3.org/2001/XMLSchema#short" = parse_rdf,
	xsd_types::Byte: "http://www.w3.org/2001/XMLSchema#byte" = parse_rdf,
	xsd_types::NonNegativeInteger: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" = parse_rdf,
	xsd_types::PositiveInteger: "http://www.w3.org/2001/XMLSchema#positiveInteger" = parse_rdf,
	xsd_types::UnsignedLong: "http://www.w3.org/2001/XMLSchema#unsignedLong" = parse_rdf,
	xsd_types::UnsignedInt: "http://www.w3.org/2001/XMLSchema#unsignedInt" = parse_rdf,
	xsd_types::UnsignedShort: "http://www.w3.org/2001/XMLSchema#unsignedShort" = parse_rdf,
	xsd_types::UnsignedByte: "http://www.w3.org/2001/XMLSchema#unsignedByte" = parse_rdf,
	xsd_types::NonPositiveInteger: "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" = parse_rdf,
	xsd_types::NegativeInteger: "http://www.w3.org/2001/XMLSchema#negativeInteger" = parse_rdf,
	xsd_types::Double: "http://www.w3.org/2001/XMLSchema#double" = parse_rdf,
	xsd_types::Float: "http://www.w3.org/2001/XMLSchema#float" = parse_rdf,
	xsd_types::String: "http://www.w3.org/2001/XMLSchema#string" = parse_rdf,
	xsd_types::Base64BinaryBuf: "http://www.w3.org/2001/XMLSchema#base64Binary" = parse_rdf,
	xsd_types::HexBinaryBuf: "http://www.w3.org/2001/XMLSchema#hexBinary" = parse_rdf,
	::chrono::NaiveDate: "http://www.w3.org/2001/XMLSchema#date" = from_str,
	::chrono::DateTime<::chrono::Utc>: "http://www.w3.org/2001/XMLSchema#dateTime" = from_str,
	iref::IriBuf: "http://www.w3.org/2001/XMLSchema#anyURI" = from_str
}
