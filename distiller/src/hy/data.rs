use iref::Iri;
use rdf_types::{
	Interpretation, IriVocabulary, LanguageTagVocabulary, ReverseIriInterpretation,
	ReverseLiteralInterpretation, Vocabulary,
};
use treeldr::{
	layout::{DataLayout, DataLayoutType},
	pattern::Substitution,
	utils::QuadsExt,
	value::Number,
	Matching, Ref, TypedLiteral,
};

use super::Error;

pub type RdfLiteralType<V> =
	rdf_types::literal::Type<<V as IriVocabulary>::Iri, <V as LanguageTagVocabulary>::LanguageTag>;

pub fn hydrate_data<V, I: Interpretation, D>(
	vocabulary: &V,
	interpretation: &I,
	dataset: &D,
	current_graph: Option<&I::Resource>,
	layout_ref: Ref<DataLayoutType, I::Resource>,
	layout: &DataLayout<I::Resource>,
	inputs: &[I::Resource],
) -> Result<TypedLiteral<I::Resource>, Error>
where
	V: Vocabulary<Type = RdfLiteralType<V>>,
	V::Iri: PartialEq,
	V::Value: AsRef<str>,
	I: ReverseIriInterpretation<Iri = V::Iri> + ReverseLiteralInterpretation<Literal = V::Literal>,
	I::Resource: Clone + PartialEq,
	D: grdf::Dataset<
		Subject = I::Resource,
		Predicate = I::Resource,
		Object = I::Resource,
		GraphLabel = I::Resource,
	>,
{
	match layout {
		DataLayout::Unit(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);

			Matching::new(
				dataset,
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph),
			)
			.into_required_unique()?;

			Ok(TypedLiteral::Unit(layout_ref.cast()))
		}
		DataLayout::Boolean(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);

			let substitution = Matching::new(
				dataset,
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph),
			)
			.into_required_unique()?;

			let resource = layout
				.resource
				.apply(&substitution)
				.into_resource()
				.unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_() {
					rdf_types::literal::Type::Any(i) => i,
					rdf_types::literal::Type::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.datatype).any(|j| i == j) {
					let v = hydrate_boolean_value(
						literal.value().as_ref(),
						vocabulary.iri(i).unwrap(),
					)?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => Ok(TypedLiteral::Boolean(value, layout_ref.casted())),
				None => {
					todo!() // No matching literal representation found
				}
			}
		}
		DataLayout::Number(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);

			let substitution = Matching::new(
				dataset,
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph),
			)
			.into_required_unique()?;

			let resource = layout
				.resource
				.apply(&substitution)
				.into_resource()
				.unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_() {
					rdf_types::literal::Type::Any(i) => i,
					rdf_types::literal::Type::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.datatype).any(|j| i == j) {
					let v =
						hydrate_number_value(literal.value().as_ref(), vocabulary.iri(i).unwrap())?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => Ok(TypedLiteral::Number(value, layout_ref.casted())),
				None => {
					todo!() // No matching literal representation found
				}
			}
		}
		DataLayout::ByteString(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);

			let substitution = Matching::new(
				dataset,
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph),
			)
			.into_required_unique()?;

			let resource = layout
				.resource
				.apply(&substitution)
				.into_resource()
				.unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_() {
					rdf_types::literal::Type::Any(i) => i,
					rdf_types::literal::Type::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.datatype).any(|j| i == j) {
					let v = hydrate_byte_string_value(
						literal.value().as_ref(),
						vocabulary.iri(i).unwrap(),
					)?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => Ok(TypedLiteral::ByteString(value, layout_ref.casted())),
				None => {
					todo!() // No matching literal representation found
				}
			}
		}
		DataLayout::TextString(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);

			let substitution = Matching::new(
				dataset,
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph),
			)
			.into_required_unique()?;

			let resource = layout
				.resource
				.apply(&substitution)
				.into_resource()
				.unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_() {
					rdf_types::literal::Type::Any(i) => i,
					rdf_types::literal::Type::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.datatype).any(|j| i == j) {
					let v = hydrate_text_string_value(
						literal.value().as_ref(),
						vocabulary.iri(i).unwrap(),
					)?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => Ok(TypedLiteral::TextString(value, layout_ref.casted())),
				None => {
					todo!() // No matching literal representation found
				}
			}
		}
	}
}

fn hydrate_boolean_value(value: &str, type_: &Iri) -> Result<bool, Error> {
	use xsd_types::ParseRdf;
	if type_ == xsd_types::XSD_BOOLEAN {
		bool::parse_rdf(value).map_err(|_| todo!())
	} else {
		todo!() // unknown boolean type.
	}
}

fn hydrate_number_value(_value: &str, _type_: &Iri) -> Result<Number, Error> {
	todo!()
}

fn hydrate_byte_string_value(_value: &str, _type_: &Iri) -> Result<Vec<u8>, Error> {
	todo!()
}

fn hydrate_text_string_value(value: &str, type_: &Iri) -> Result<String, Error> {
	if type_ == xsd_types::XSD_STRING {
		Ok(value.to_owned())
	} else {
		todo!() // unknown string type.
	}
}
