use rdf_types::{
	literal::Type, IriVocabulary, LanguageTagVocabulary, LiteralVocabulary,
	ReverseIriInterpretation,
};

use crate::{distill::RdfContextMut, value::Number};

use super::{Error, RdfLiteral, RdfLiteralType};

pub fn dehydrate_boolean<V, I>(
	rdf: &RdfContextMut<V, I>,
	value: bool,
	type_: &I::Resource,
) -> Result<RdfLiteral<V>, Error>
where
	V: IriVocabulary + LanguageTagVocabulary + LiteralVocabulary<Type = RdfLiteralType<V>>,
	V::Iri: Clone,
	V::Value: From<String>,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	for i in rdf.interpretation.iris_of(type_) {
		let iri = rdf.vocabulary.iri(i).unwrap();
		if iri == xsd_types::XSD_BOOLEAN {
			return Ok(rdf_types::Literal::new(
				xsd_types::lexical::BooleanBuf::from(value)
					.into_string()
					.into(),
				Type::Any(i.clone()),
			));
		}
	}

	todo!()
}

pub fn dehydrate_number<V, I>(
	rdf: &RdfContextMut<V, I>,
	value: &Number,
	type_: &I::Resource,
) -> Result<RdfLiteral<V>, Error>
where
	V: IriVocabulary + LanguageTagVocabulary + LiteralVocabulary<Type = RdfLiteralType<V>>,
	V::Iri: Clone,
	V::Value: From<String>,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	for i in rdf.interpretation.iris_of(type_) {
		let iri = rdf.vocabulary.iri(i).unwrap();
		if let Some(xsd_types::Datatype::Decimal(_)) = xsd_types::Datatype::from_iri(iri) {
			if let Ok(value) = xsd_types::Decimal::try_from(value.clone()) {
				// TODO better support for XSD decimal datatype.
				return Ok(rdf_types::Literal::new(
					value.to_string().into(),
					Type::Any(i.clone()),
				));
			}
		}
	}

	todo!()
}

pub fn dehydrate_byte_string<V, I>(
	_rdf: &RdfContextMut<V, I>,
	_value: &[u8],
	_type_: &I::Resource,
) -> Result<RdfLiteral<V>, Error>
where
	V: IriVocabulary + LanguageTagVocabulary + LiteralVocabulary<Type = RdfLiteralType<V>>,
	V::Iri: Clone,
	V::Value: From<String>,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	todo!()
}

pub fn dehydrate_text_string<V, I>(
	rdf: &RdfContextMut<V, I>,
	value: &str,
	type_: &I::Resource,
) -> Result<RdfLiteral<V>, Error>
where
	V: IriVocabulary + LanguageTagVocabulary + LiteralVocabulary<Type = RdfLiteralType<V>>,
	V::Iri: Clone,
	V::Value: From<String>,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	match rdf.interpretation.iris_of(type_).next() {
		Some(i) => Ok(rdf_types::Literal::new(
			value.to_owned().into(),
			Type::Any(i.clone()),
		)),
		None => todo!(),
	}
}
