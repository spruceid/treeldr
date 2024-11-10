use std::borrow::Cow;

use iref::Iri;
use rdf_types::{
	dataset::PatternMatchingDataset,
	interpretation::{IriInterpretation, ReverseIriInterpretation, ReverseLiteralInterpretation},
	vocabulary::{IriVocabulary, LiteralVocabulary},
	LiteralTypeRef, RDF_LANG_STRING,
};
use xsd_types::XSD_STRING;

use crate::{
	eval::{EvalError, MissingData, ReverseScope},
	rdf::{RdfContext, RdfContextMut},
	ty::BuiltInType,
	DatasetPattern, TypeRef, Value,
};

use super::Signature;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltInFunction {
	Get,
	IriOf,
	BoolOf,
	TypedBoolOf,
	NumberOf,
	TypedNumberOf,
	IntOf,
	TypedIntOf,
	UintOf,
	TypedUintOf,
	StringOf,
	TypedStringOf,
}

impl BuiltInFunction {
	pub fn signature<R>(&self, ty: impl Fn(BuiltInType) -> TypeRef) -> Signature<R> {
		match self {
			Self::Get => Signature {
				args: vec![ty(BuiltInType::Resource), ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Resource),
				dataset: DatasetPattern::new(),
			},
			Self::IriOf => Signature {
				args: vec![ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::TextString),
				dataset: DatasetPattern::new(),
			},
			Self::BoolOf => Signature {
				args: vec![ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Boolean),
				dataset: DatasetPattern::new(),
			},
			Self::TypedBoolOf => Signature {
				args: vec![ty(BuiltInType::Resource), ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Boolean),
				dataset: DatasetPattern::new(),
			},
			Self::NumberOf => Signature {
				args: vec![ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Number),
				dataset: DatasetPattern::new(),
			},
			Self::TypedNumberOf => Signature {
				args: vec![ty(BuiltInType::Resource), ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Number),
				dataset: DatasetPattern::new(),
			},
			Self::IntOf => Signature {
				args: vec![ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Int),
				dataset: DatasetPattern::new(),
			},
			Self::TypedIntOf => Signature {
				args: vec![ty(BuiltInType::Resource), ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Int),
				dataset: DatasetPattern::new(),
			},
			Self::UintOf => Signature {
				args: vec![ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Uint),
				dataset: DatasetPattern::new(),
			},
			Self::TypedUintOf => Signature {
				args: vec![ty(BuiltInType::Resource), ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::Uint),
				dataset: DatasetPattern::new(),
			},
			Self::StringOf => Signature {
				args: vec![ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::TextString),
				dataset: DatasetPattern::new(),
			},
			Self::TypedStringOf => Signature {
				args: vec![ty(BuiltInType::Resource), ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::TextString),
				dataset: DatasetPattern::new(),
			},
		}
	}

	pub fn eval<R: Ord + Clone>(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		args: &[Value<R>],
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Get => {
				let prop = args[0].as_resource().unwrap();
				let subject = args[1].as_resource().unwrap();

				let mut objects = dataset.quad_objects(None, subject, prop);

				match objects.next() {
					Some(object) => {
						if objects.next().is_some() {
							Err(EvalError::Ambiguity)
						} else {
							Ok(Value::Resource(object.clone()))
						}
					}
					None => Err(EvalError::MissingData(MissingData::Empty)),
				}
			}
			Self::IriOf => {
				let r = args[0].as_resource().unwrap();

				let mut value = None;
				for i in rdf.interpretation().iris_of(r) {
					let iri = rdf.vocabulary().iri(i).unwrap();
					if let Some(old_value) = value.replace(iri) {
						return Err(EvalError::AmbiguousIriRepresentation(
							old_value.to_owned(),
							iri.to_owned(),
						));
					}
				}

				match value {
					Some(value) => Ok(Value::string(value.as_str().to_owned())),
					None => Err(EvalError::MissingData(
						MissingData::NoMatchingIriRepresentation,
					)),
				}
			}
			Self::BoolOf => {
				todo!()
			}
			Self::TypedBoolOf => {
				todo!()
			}
			Self::NumberOf => {
				todo!()
			}
			Self::TypedNumberOf => {
				todo!()
			}
			Self::IntOf => {
				todo!()
			}
			Self::TypedIntOf => {
				todo!()
			}
			Self::UintOf => {
				todo!()
			}
			Self::TypedUintOf => {
				todo!()
			}
			Self::StringOf => {
				let r = args[0].as_resource().unwrap();

				find_literal_lexical(rdf, r, XSD_STRING)
					.map(ToOwned::to_owned)
					.map(Value::string)
			}
			Self::TypedStringOf => {
				let ty = args[0].as_resource().unwrap();
				let r = args[1].as_resource().unwrap();

				find_literal_interpreted(rdf, r, ty)
					.map(ToOwned::to_owned)
					.map(Value::string)
			}
		}
	}

	pub fn eval_inverse<R>(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		_scope: &mut ReverseScope<R>,
		output: &Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>> {
		match self {
			Self::Get => {
				todo!()
			}
			Self::IriOf => {
				let value = output.as_str().unwrap();
				let iri = Iri::new(value).map_err(|_| EvalError::InvalidValue)?;
				let r = rdf.insert_iri(Cow::Borrowed(iri));
				Ok(vec![Value::Resource(r)])
			}
			Self::BoolOf => {
				todo!()
			}
			Self::TypedBoolOf => {
				todo!()
			}
			Self::NumberOf => {
				todo!()
			}
			Self::TypedNumberOf => {
				todo!()
			}
			Self::IntOf => {
				todo!()
			}
			Self::TypedIntOf => {
				todo!()
			}
			Self::UintOf => {
				todo!()
			}
			Self::TypedUintOf => {
				todo!()
			}
			Self::StringOf => {
				let value = output.as_str().unwrap();
				let r = rdf.insert_literal(value.to_owned(), Cow::Borrowed(XSD_STRING));
				Ok(vec![Value::Resource(r)])
			}
			Self::TypedStringOf => {
				todo!()
			}
		}
	}
}

fn find_literal_lexical<'a, R: PartialEq>(
	rdf: &'a impl RdfContext<R>,
	resource: &'a R,
	expected_ty: &Iri,
) -> Result<&'a str, EvalError<R>> {
	let mut value = None;
	for l in rdf.interpretation().literals_of(resource) {
		let literal = rdf.vocabulary().literal(l).unwrap();

		let type_resource = match literal.type_ {
			LiteralTypeRef::Any(i) => rdf.vocabulary().iri(i).unwrap(),
			LiteralTypeRef::LangString(_) => RDF_LANG_STRING,
		};

		if type_resource == expected_ty {
			if let Some(old_value) = value.replace(literal.value) {
				return Err(EvalError::AmbiguousLiteralRepresentation(
					old_value.to_owned(),
					literal.value.to_owned(),
				));
			}
		}
	}

	match value {
		Some(value) => Ok(value),
		None => Err(EvalError::MissingData(
			MissingData::NoMatchingLiteralRepresentation,
		)),
	}
}

fn find_literal_interpreted<'a, R: PartialEq>(
	rdf: &'a impl RdfContext<R>,
	resource: &'a R,
	expected_ty: &R,
) -> Result<&'a str, EvalError<R>> {
	let mut value = None;
	for l in rdf.interpretation().literals_of(resource) {
		let literal = rdf.vocabulary().literal(l).unwrap();

		let type_resource = match literal.type_ {
			LiteralTypeRef::Any(i) => rdf.interpretation().iri_interpretation(i),
			LiteralTypeRef::LangString(_) => rdf
				.interpretation()
				.lexical_iri_interpretation(rdf.vocabulary(), RDF_LANG_STRING),
		};

		if let Some(type_resource) = type_resource {
			if type_resource == *expected_ty {
				if let Some(old_value) = value.replace(literal.value) {
					return Err(EvalError::AmbiguousLiteralRepresentation(
						old_value.to_owned(),
						literal.value.to_owned(),
					));
				}
			}
		}
	}

	match value {
		Some(value) => Ok(value),
		None => Err(EvalError::MissingData(
			MissingData::NoMatchingLiteralRepresentation,
		)),
	}
}
