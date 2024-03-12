use rdf_types::{
	interpretation::ReverseIriInterpretation,
	vocabulary::{IriVocabulary, LiteralVocabulary},
	LiteralType,
};

use crate::{distill::RdfContextMut, value::Number};

use super::{Error, RdfLiteral};

pub fn dehydrate_boolean<V, I, Q>(
	rdf: &RdfContextMut<V, I>,
	value: bool,
	type_: &I::Resource,
) -> Result<RdfLiteral<V>, Error<Q>>
where
	V: LiteralVocabulary,
	V::Iri: Clone,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	for i in rdf.interpretation.iris_of(type_) {
		let iri = rdf.vocabulary.iri(i).unwrap();
		if iri == xsd_types::XSD_BOOLEAN {
			return Ok(rdf_types::Literal::new(
				xsd_types::lexical::BooleanBuf::from(value).into_string(),
				LiteralType::Any(i.clone()),
			));
		}
	}

	todo!()
}

pub fn dehydrate_number<V, I, Q>(
	rdf: &RdfContextMut<V, I>,
	value: &Number,
	type_: &I::Resource,
) -> Result<RdfLiteral<V>, Error<Q>>
where
	V: LiteralVocabulary,
	V::Iri: Clone,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	for i in rdf.interpretation.iris_of(type_) {
		let iri = rdf.vocabulary.iri(i).unwrap();
		if let Some(xsd_types::Datatype::Decimal(_)) = xsd_types::Datatype::from_iri(iri) {
			if let Ok(decimal) = xsd_types::Decimal::try_from(value.as_big_rational().clone()) {
				// TODO better support for XSD decimal datatype.
				return Ok(rdf_types::Literal::new(
					decimal.to_string(),
					LiteralType::Any(i.clone()),
				));
			}
		}
	}

	todo!()
}

pub fn dehydrate_byte_string<V, I, Q>(
	_rdf: &RdfContextMut<V, I>,
	_value: &[u8],
	_type_: &I::Resource,
) -> Result<RdfLiteral<V>, Error<Q>>
where
	V: LiteralVocabulary,
	V::Iri: Clone,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	todo!()
}

pub fn dehydrate_text_string<V, I, Q>(
	rdf: &RdfContextMut<V, I>,
	value: &str,
	type_: &I::Resource,
) -> Result<RdfLiteral<V>, Error<Q>>
where
	V: LiteralVocabulary,
	V::Iri: Clone,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	match rdf.interpretation.iris_of(type_).next() {
		Some(i) => Ok(rdf_types::Literal::new(
			value.to_owned(),
			LiteralType::Any(i.clone()),
		)),
		None => todo!(),
	}
}
