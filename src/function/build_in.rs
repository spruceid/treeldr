use std::borrow::Cow;

use iref::Iri;
use rdf_types::{
	dataset::PatternMatchingDataset,
	interpretation::{ReverseIriInterpretation, ReverseLiteralInterpretation},
	vocabulary::{IriVocabulary, LiteralVocabulary},
	LiteralTypeRef,
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
	IriOf,
	StringOf,
}

impl BuiltInFunction {
	pub fn signature<R>(&self, ty: impl Fn(BuiltInType) -> TypeRef) -> Signature<R> {
		match self {
			Self::IriOf => Signature {
				args: vec![ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::TextString),
				dataset: DatasetPattern::new(),
			},
			Self::StringOf => Signature {
				args: vec![ty(BuiltInType::Resource)],
				return_type: ty(BuiltInType::TextString),
				dataset: DatasetPattern::new(),
			},
		}
	}

	pub fn eval<R: Ord + Clone>(
		&self,
		rdf: &impl RdfContext<R>,
		_dataset: &impl PatternMatchingDataset<Resource = R>,
		args: &[Value<R>],
	) -> Result<Value<R>, EvalError<R>> {
		match self {
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
			Self::StringOf => {
				let r = args[0].as_resource().unwrap();

				let mut value = None;
				for l in rdf.interpretation().literals_of(r) {
					let literal = rdf.vocabulary().literal(l).unwrap();
					if let LiteralTypeRef::Any(ty) = literal.type_ {
						if rdf.vocabulary().iri(ty).unwrap() == XSD_STRING {
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
					Some(value) => Ok(Value::string(value.to_owned())),
					None => Err(EvalError::MissingData(
						MissingData::NoMatchingLiteralRepresentation,
					)),
				}
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
			Self::IriOf => {
				let value = output.as_str().unwrap();
				let iri = Iri::new(value).map_err(|_| EvalError::InvalidValue)?;
				let r = rdf.insert_iri(Cow::Borrowed(iri));
				Ok(vec![Value::Resource(r)])
			}
			Self::StringOf => {
				let value = output.as_str().unwrap();
				let r = rdf.insert_literal(value.to_owned(), Cow::Borrowed(XSD_STRING));
				Ok(vec![Value::Resource(r)])
			}
		}
	}
}
