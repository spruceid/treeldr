use rdf_types::{VocabularyMut, InterpretationMut, Literal, literal, XSD_STRING, ReverseTermInterpretationMut, TermInterpretationMut};

use crate::{SerializeLd, RdfType};

impl<V, I> SerializeLd<1, V, I> for String
where
	V: VocabularyMut<Value = String, Type = RdfType<V>>,
	I: InterpretationMut<V> + TermInterpretationMut<V::Iri, V::BlankId, V::Literal> + ReverseTermInterpretationMut<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	I::Resource: Clone + Ord
{
	fn serialize_ld_with(
		&self,
		rdf: &mut crate::RdfContextMut<V, I>,
		inputs: &[<I as rdf_types::Interpretation>::Resource; 1],
		_current_graph: Option<&<I as rdf_types::Interpretation>::Resource>,
		_output: &mut grdf::BTreeDataset<<I as rdf_types::Interpretation>::Resource>
	) -> Result<(), crate::SerializeError> {
		let l = rdf.vocabulary_literal(Literal::new(
			self.as_str(),
			literal::Type::Any(XSD_STRING)
		));

		rdf.interpretation.assign_literal(inputs[0].clone(), l);
		Ok(())
	}
}