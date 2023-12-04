use rdf_types::{
	literal, InterpretationMut, Literal, ReverseTermInterpretation, ReverseTermInterpretationMut,
	TermInterpretation, TermInterpretationMut, Vocabulary, VocabularyMut, XSD_STRING,
};

use crate::{
	DeserializeError, DeserializeLd, RdfContext, RdfContextMut, RdfType, SerializeError,
	SerializeLd,
};

impl<V, I> SerializeLd<1, V, I> for String
where
	V: VocabularyMut<Value = String, Type = RdfType<V>>,
	I: InterpretationMut<V>
		+ TermInterpretationMut<V::Iri, V::BlankId, V::Literal>
		+ ReverseTermInterpretationMut<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	I::Resource: Clone + Ord,
{
	fn serialize_ld_with(
		&self,
		rdf: &mut RdfContextMut<V, I>,
		inputs: &[<I as rdf_types::Interpretation>::Resource; 1],
		_current_graph: Option<&<I as rdf_types::Interpretation>::Resource>,
		_output: &mut grdf::BTreeDataset<<I as rdf_types::Interpretation>::Resource>,
	) -> Result<(), SerializeError> {
		let l = rdf.vocabulary_literal(Literal::new(self.as_str(), literal::Type::Any(XSD_STRING)));

		rdf.interpretation.assign_literal(inputs[0].clone(), l);
		Ok(())
	}
}

impl<V, I> DeserializeLd<1, V, I> for String
where
	V: Vocabulary<Value = String, Type = RdfType<V>>,
	I: TermInterpretation<V::Iri, V::BlankId, V::Literal>
		+ ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	I::Resource: Clone + Ord,
{
	fn deserialize_ld_with<D>(
		rdf: RdfContext<V, I>,
		_dataset: &D,
		_graph: Option<&I::Resource>,
		inputs: &[I::Resource; 1],
	) -> Result<Self, DeserializeError>
	where
		D: grdf::Dataset<
			Subject = I::Resource,
			Predicate = I::Resource,
			Object = I::Resource,
			GraphLabel = I::Resource,
		>,
	{
		for l in rdf.interpretation.literals_of(&inputs[0]) {
			let literal = rdf.vocabulary.literal(l).unwrap();
			if let literal::Type::Any(i) = literal.type_() {
				let iri = rdf.vocabulary.iri(i).unwrap();
				if iri == XSD_STRING {
					return Ok(literal.value().to_string());
				}
			}
		}

		Err(DeserializeError::MissingData)
	}
}
