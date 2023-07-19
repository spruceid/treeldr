use iref::IriBuf;
use json_ld::rdf::XSD_STRING;
use rdf_types::{
	IriVocabularyMut, LanguageTagVocabulary, LanguageTagVocabularyMut, Literal,
	LiteralInterpretationMut, LiteralVocabulary, LiteralVocabularyMut,
};
use static_iref::iri;

use crate::rdf::{LiteralValue, ValuesOnly};

use super::QuadsAndValues;

pub trait AsLiteral<V: LiteralVocabulary> {
	fn rdf_literal_value(&self, vocabulary: &mut V) -> V::Literal;
}

macro_rules! impl_as_literal {
	{ $($ty:ty : $rdf_ty:tt),* } => {
		$(
			impl<V> AsLiteral<V> for $ty
			where
				V: IriVocabularyMut + LanguageTagVocabularyMut + LiteralVocabularyMut<Type = rdf_types::literal::Type<V::Iri, V::LanguageTag>>,
				V::Value: From<String>
			{
				fn rdf_literal_value(&self, namespace: &mut V) -> V::Literal {
					let l = Literal::new(
						self.to_string().into(),
						rdf_types::literal::Type::Any(namespace.insert(iri!($rdf_ty)))
					);

					namespace.insert_owned_literal(l)
				}
			}

			impl<V: LiteralVocabulary, N: LiteralInterpretationMut<V::Literal>> QuadsAndValues<V, N> for $ty
			where
				Self: AsLiteral<V>,
			{
				type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self>> where Self: 'a, N::Resource: 'a;

				fn unbound_rdf_quads_and_values<'a>(
					&'a self,
					_vocabulary: &mut V,
					_interpretation: &mut N,
				) -> (Option<N::Resource>, Self::QuadsAndValues<'a>)
				where
					N::Resource: 'a
				{
					(None, ValuesOnly::new(LiteralValue::new(self)))
				}
			}
		)*
	};
}

impl_as_literal! {
	xsd_types::Boolean: "http://www.w3.org/2001/XMLSchema#boolean",
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
	xsd_types::Base64BinaryBuf: "http://www.w3.org/2001/XMLSchema#base64Binary",
	xsd_types::HexBinaryBuf: "http://www.w3.org/2001/XMLSchema#hexBinary"
}

impl<
		V: IriVocabularyMut
			+ LanguageTagVocabulary
			+ LiteralVocabularyMut<Type = rdf_types::literal::Type<V::Iri, V::LanguageTag>>,
	> AsLiteral<V> for String
where
	V::Value: From<String>,
{
	fn rdf_literal_value(&self, vocabulary: &mut V) -> V::Literal {
		let l = Literal::new(
			self.to_owned().into(),
			rdf_types::literal::Type::Any(vocabulary.insert(XSD_STRING)),
		);
		vocabulary.insert_owned_literal(l)
	}
}

impl<V: LiteralVocabulary, I: LiteralInterpretationMut<V::Literal>> QuadsAndValues<V, I> for String
where
	Self: AsLiteral<V>,
{
	type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self>> where Self: 'a, I::Resource: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> (Option<I::Resource>, Self::QuadsAndValues<'a>)
	where
		I::Resource: 'a,
	{
		(None, ValuesOnly::new(LiteralValue::new(self)))
	}
}

impl<V: LiteralVocabulary, I: LiteralInterpretationMut<V::Literal>> QuadsAndValues<V, I> for IriBuf
where
	Self: AsLiteral<V>,
{
	type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self>> where Self: 'a, I::Resource: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> (Option<I::Resource>, Self::QuadsAndValues<'a>)
	where
		I::Resource: 'a,
	{
		(None, ValuesOnly::new(LiteralValue::new(self)))
	}
}

impl<V: LiteralVocabulary, I: LiteralInterpretationMut<V::Literal>> QuadsAndValues<V, I>
	for chrono::NaiveDate
where
	Self: AsLiteral<V>,
{
	type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self>> where Self: 'a, I::Resource: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> (Option<I::Resource>, Self::QuadsAndValues<'a>)
	where
		I::Resource: 'a,
	{
		(None, ValuesOnly::new(LiteralValue::new(self)))
	}
}

impl<V: LiteralVocabulary, I: LiteralInterpretationMut<V::Literal>> QuadsAndValues<V, I>
	for chrono::DateTime<chrono::Utc>
where
	Self: AsLiteral<V>,
{
	type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self>> where Self: 'a, I::Resource: 'a;

	fn unbound_rdf_quads_and_values<'a>(
		&'a self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> (Option<I::Resource>, Self::QuadsAndValues<'a>)
	where
		I::Resource: 'a,
	{
		(None, ValuesOnly::new(LiteralValue::new(self)))
	}
}
