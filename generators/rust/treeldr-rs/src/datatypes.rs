use rdf_types::{
	literal, InterpretationMut, Literal, ReverseTermInterpretation, ReverseTermInterpretationMut,
	TermInterpretation, TermInterpretationMut, Vocabulary, VocabularyMut, XSD_STRING,
};

use crate::{
	DeserializeError, DeserializeLd, RdfContext, RdfContextMut, RdfType, SerializeError,
	SerializeLd,
};

impl<V, I> SerializeLd<1, V, I> for rdf_types::Id
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

impl<V, I> DeserializeLd<1, V, I> for rdf_types::Id
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
		let mut id = None;

		for i in rdf.interpretation.iris_of(&inputs[0]) {
			let iri = rdf.vocabulary.iri(i).unwrap();
			if id.replace(Self::Iri(iri.to_owned())).is_some() {
				return Err(DeserializeError::AmbiguousId);
			}
		}

		if id.is_none() {
			for b in rdf.interpretation.blank_ids_of(&inputs[0]) {
				let blank_id = rdf.vocabulary.blank_id(b).unwrap();
				if id.replace(Self::Blank(blank_id.to_owned())).is_some() {
					return Err(DeserializeError::AmbiguousId);
				}
			}
		}

		match id {
			Some(id) => Ok(id),
			None => Err(DeserializeError::MissingId),
		}
	}
}

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

macro_rules! xsd_datatypes {
	($($ty:ident : $xsd_iri:ident),*) => {
		$(
			impl<V, I> SerializeLd<1, V, I> for $ty
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
					let l = rdf.vocabulary_literal_owned(Literal::new(self.to_string(), literal::Type::Any(xsd_types::$xsd_iri.to_owned())));
					rdf.interpretation.assign_literal(inputs[0].clone(), l);
					Ok(())
				}
			}

			impl<V, I> DeserializeLd<1, V, I> for $ty
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
					use xsd_types::ParseRdf;
					let mut result = None;
					let mut has_literal = false;
					for l in rdf.interpretation.literals_of(&inputs[0]) {
						has_literal = true;
						let literal = rdf.vocabulary.literal(l).unwrap();
						if let literal::Type::Any(i) = literal.type_() {
							let iri = rdf.vocabulary.iri(i).unwrap();
							if iri == xsd_types::$xsd_iri {
								match Self::parse_rdf(literal.value()) {
									Ok(value) => {
										if result.replace(value).is_some() {
											return Err(DeserializeError::AmbiguousLiteralValue)
										}
									},
									Err(_) => {
										return Err(DeserializeError::InvalidLiteralValue)
									}
								}
							}
						}
					}

					match result {
						Some(r) => Ok(r),
						None => if has_literal {
							Err(DeserializeError::LiteralTypeMismatch)
						} else {
							Err(DeserializeError::ExpectedLiteral)
						}
					}
				}
			}
		)*
	};
}

xsd_datatypes! {
	u8: XSD_UNSIGNED_BYTE,
	u16: XSD_UNSIGNED_SHORT,
	u32: XSD_UNSIGNED_INT,
	u64: XSD_UNSIGNED_LONG,
	i8: XSD_BYTE,
	i16: XSD_SHORT,
	i32: XSD_INT,
	i64: XSD_LONG
}
