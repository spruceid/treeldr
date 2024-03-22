use crate::{
	layout::{DataLayout, DataLayoutType},
	pattern::Substitution,
	utils::QuadsExt,
	value::Number,
	Matching, Ref, TypedLiteral,
};
use iref::Iri;
use rdf_types::{
	dataset::{PatternMatchingDataset, TraversableDataset},
	interpretation::{ReverseIriInterpretation, ReverseLiteralInterpretation},
	Interpretation, LiteralTypeRef, Vocabulary,
};
use xsd_types::{lexical::Lexical, ParseXsd};

use super::{DataFragment, Error, MatchingForFragment};

pub fn hydrate_data<V, I: Interpretation, D>(
	vocabulary: &V,
	interpretation: &I,
	dataset: &D,
	current_graph: Option<&I::Resource>,
	layout_ref: Ref<DataLayoutType, I::Resource>,
	layout: &DataLayout<I::Resource>,
	inputs: &[I::Resource],
) -> Result<TypedLiteral<I::Resource>, Error<I::Resource>>
where
	V: Vocabulary,
	V::Iri: PartialEq,
	I: ReverseIriInterpretation<Iri = V::Iri> + ReverseLiteralInterpretation<Literal = V::Literal>,
	I::Resource: Clone + PartialEq,
	D: PatternMatchingDataset<Resource = I::Resource>,
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
			.into_required_unique()
			.for_fragment(|| DataFragment::Discriminant(layout_ref.clone().cast()))?;

			Ok(TypedLiteral::Unit(layout.const_.clone(), layout_ref.cast()))
		}
		DataLayout::Boolean(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);

			let substitution = Matching::new(
				dataset,
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph),
			)
			.into_required_unique()
			.for_fragment(|| DataFragment::Discriminant(layout_ref.clone().cast()))?;

			let resource = layout
				.resource
				.apply(&substitution)
				.into_resource()
				.unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_ {
					LiteralTypeRef::Any(i) => i,
					LiteralTypeRef::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.datatype).any(|j| i == j) {
					let v = hydrate_boolean_value(literal.value, vocabulary.iri(i).unwrap())?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => Ok(TypedLiteral::Boolean(value, layout_ref.casted())),
				None => Err(Error::NoMatchingLiteral),
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
			.into_required_unique()
			.for_fragment(|| DataFragment::Discriminant(layout_ref.clone().cast()))?;

			let resource = layout
				.resource
				.apply(&substitution)
				.into_resource()
				.unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_ {
					LiteralTypeRef::Any(i) => i,
					LiteralTypeRef::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.datatype).any(|j| i == j) {
					let v = hydrate_number_value(literal.value, vocabulary.iri(i).unwrap())?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => Ok(TypedLiteral::Number(value, layout_ref.casted())),
				None => Err(Error::NoMatchingLiteral),
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
			.into_required_unique()
			.for_fragment(|| DataFragment::Discriminant(layout_ref.clone().cast()))?;

			let resource = layout
				.resource
				.apply(&substitution)
				.into_resource()
				.unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_ {
					LiteralTypeRef::Any(i) => i,
					LiteralTypeRef::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.datatype).any(|j| i == j) {
					let v = hydrate_byte_string_value(literal.value, vocabulary.iri(i).unwrap())?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => Ok(TypedLiteral::ByteString(value, layout_ref.casted())),
				None => Err(Error::NoMatchingLiteral),
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
			.into_required_unique()
			.for_fragment(|| DataFragment::Discriminant(layout_ref.clone().cast()))?;

			let resource = layout
				.resource
				.apply(&substitution)
				.into_resource()
				.unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_ {
					LiteralTypeRef::Any(i) => i,
					LiteralTypeRef::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.datatype).any(|j| i == j) {
					let v = hydrate_text_string_value(literal.value, vocabulary.iri(i).unwrap())?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => Ok(TypedLiteral::TextString(value, layout_ref.casted())),
				None => Err(Error::NoMatchingLiteral),
			}
		}
	}
}

fn hydrate_boolean_value<R>(value: &str, type_: &Iri) -> Result<bool, Error<R>> {
	use xsd_types::Boolean;
	if type_ == xsd_types::XSD_BOOLEAN {
		Boolean::parse_xsd(value)
			.map(Boolean::into)
			.map_err(|_| todo!())
	} else {
		todo!() // unknown boolean type.
	}
}

fn hydrate_number_value<R>(value: &str, type_: &Iri) -> Result<Number, Error<R>> {
	match xsd_types::Datatype::from_iri(type_) {
		Some(xsd_types::Datatype::Decimal(_t)) => match xsd_types::lexical::Decimal::parse(value) {
			Ok(value) => {
				let decimal = value.value();
				Ok(decimal.into_big_rational().into())
			}
			Err(_) => {
				todo!()
			}
		},
		_ => Err(Error::UnknownNumberDatatype(type_.to_owned())),
	}
}

fn hydrate_byte_string_value<R>(_value: &str, _type_: &Iri) -> Result<Vec<u8>, Error<R>> {
	todo!()
}

fn hydrate_text_string_value<R>(value: &str, _type_: &Iri) -> Result<String, Error<R>> {
	Ok(value.to_string())
}
