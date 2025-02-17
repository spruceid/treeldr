use std::borrow::Cow;

use iref::Iri;
use rdf_types::{CowLiteral, CowLiteralType, Quad, RDF_LANG_STRING};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};
use xsd_types::{
	ParseXsd, XSD_BOOLEAN, XSD_DECIMAL, XSD_INTEGER, XSD_NON_NEGATIVE_INTEGER, XSD_STRING,
};

use crate::{
	// eval::{EvalError, MissingData, Scope},
	// rdf::DebugSubstitution,
	eval::{EvalContext, EvalError, MissingData, Scope},
	rdf::DebugSubstitution,
	ty::{BuiltInType, BuiltInTypeMap},
	DatasetPattern,
	TypeRef,
	Value,
};

use super::Signature;

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(ser(params(Q), seed(Q)), de(params(Q), seed(Q)))]
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
	pub fn signature<P: Copy, R>(&self, ty: &BuiltInTypeMap<TypeRef<P>>) -> Signature<P, R> {
		match self {
			Self::Get => Signature::new(
				vec![],
				vec![
					*ty.get(BuiltInType::Resource),
					*ty.get(BuiltInType::Resource),
				],
				*ty.get(BuiltInType::Resource),
				DatasetPattern::new(),
			),
			Self::IriOf => Signature::new(
				vec![],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::TextString),
				DatasetPattern::new(),
			),
			Self::BoolOf => Signature::new(
				vec![],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::Boolean),
				DatasetPattern::new(),
			),
			Self::TypedBoolOf => Signature::new(
				vec![*ty.get(BuiltInType::Resource)],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::Boolean),
				DatasetPattern::new(),
			),
			Self::NumberOf => Signature::new(
				vec![],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::Number),
				DatasetPattern::new(),
			),
			Self::TypedNumberOf => Signature::new(
				vec![*ty.get(BuiltInType::Resource)],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::Number),
				DatasetPattern::new(),
			),
			Self::IntOf => Signature::new(
				vec![],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::Int),
				DatasetPattern::new(),
			),
			Self::TypedIntOf => Signature::new(
				vec![*ty.get(BuiltInType::Resource)],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::Int),
				DatasetPattern::new(),
			),
			Self::UintOf => Signature::new(
				vec![],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::Uint),
				DatasetPattern::new(),
			),
			Self::TypedUintOf => Signature::new(
				vec![*ty.get(BuiltInType::Resource)],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::Uint),
				DatasetPattern::new(),
			),
			Self::StringOf => Signature::new(
				vec![],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::TextString),
				DatasetPattern::new(),
			),
			Self::TypedStringOf => Signature::new(
				vec![*ty.get(BuiltInType::Resource)],
				vec![*ty.get(BuiltInType::Resource)],
				*ty.get(BuiltInType::TextString),
				DatasetPattern::new(),
			),
		}
	}

	pub fn eval<R: Ord + Clone>(
		&self,
		rdf: &impl EvalContext<Resource = R>,
		args: &[Value<R>],
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Get => {
				let predicate = args[0].as_resource().unwrap();
				let subject = args[1].as_resource().unwrap();

				let mut objects = rdf.quad_objects(None, subject, predicate);

				match objects.next() {
					Some(object) => {
						if objects.next().is_some() {
							Err(EvalError::Ambiguity)
						} else {
							Ok(Value::Resource(object.clone()))
						}
					}
					None => {
						let mut substitution = DebugSubstitution::new();
						Err(EvalError::MissingData(MissingData::Empty(
							[Quad(
								substitution.insert_resource(rdf, subject),
								substitution.insert_resource(rdf, predicate),
								substitution.new_resource(),
								None,
							)]
							.into_iter()
							.collect(),
						)))
					}
				}
			}
			Self::IriOf => {
				let r = args[0].as_resource().unwrap();
				let iri = iri_of(rdf, r)?;
				Ok(Value::string(iri.into_owned().into_string()))
			}
			Self::BoolOf => {
				let r = args[0].as_resource().unwrap();
				let v = xsd_types::Boolean::parse_xsd(
					find_literal_lexical(rdf, r, XSD_BOOLEAN)?.as_ref(),
				)
				.map_err(|_| EvalError::InvalidValue)?;
				Ok(Value::boolean(v.into()))
			}
			Self::TypedBoolOf => {
				let ty = args[0].as_resource().unwrap();
				let r = args[1].as_resource().unwrap();
				let v =
					xsd_types::Boolean::parse_xsd(find_literal_interpreted(rdf, r, ty)?.as_ref())
						.map_err(|_| EvalError::InvalidValue)?;
				Ok(Value::boolean(v.into()))
			}
			Self::NumberOf => {
				let r = args[0].as_resource().unwrap();
				let v = xsd_types::Decimal::parse_xsd(
					find_literal_lexical(rdf, r, XSD_DECIMAL)?.as_ref(),
				)
				.map_err(|_| EvalError::InvalidValue)?;
				Ok(Value::number(v.into()))
			}
			Self::TypedNumberOf => {
				let ty = args[0].as_resource().unwrap();
				let r = args[1].as_resource().unwrap();
				let v =
					xsd_types::Decimal::parse_xsd(find_literal_interpreted(rdf, r, ty)?.as_ref())
						.map_err(|_| EvalError::InvalidValue)?;
				Ok(Value::number(v.into()))
			}
			Self::IntOf => {
				let r = args[0].as_resource().unwrap();
				let v = xsd_types::Integer::parse_xsd(
					find_literal_lexical(rdf, r, XSD_INTEGER)?.as_ref(),
				)
				.map_err(|_| EvalError::InvalidValue)?;
				Ok(Value::number(v.into()))
			}
			Self::TypedIntOf => {
				let ty = args[0].as_resource().unwrap();
				let r = args[1].as_resource().unwrap();
				let v =
					xsd_types::Integer::parse_xsd(find_literal_interpreted(rdf, r, ty)?.as_ref())
						.map_err(|_| EvalError::InvalidValue)?;
				Ok(Value::number(v.into()))
			}
			Self::UintOf => {
				let r = args[0].as_resource().unwrap();
				let v = xsd_types::NonNegativeInteger::parse_xsd(
					find_literal_lexical(rdf, r, XSD_NON_NEGATIVE_INTEGER)?.as_ref(),
				)
				.map_err(|_| EvalError::InvalidValue)?;
				Ok(Value::number(v.into()))
			}
			Self::TypedUintOf => {
				let ty = args[0].as_resource().unwrap();
				let r = args[1].as_resource().unwrap();
				let v = xsd_types::NonNegativeInteger::parse_xsd(
					find_literal_interpreted(rdf, r, ty)?.as_ref(),
				)
				.map_err(|_| EvalError::InvalidValue)?;
				Ok(Value::number(v.into()))
			}
			Self::StringOf => {
				let r = args[0].as_resource().unwrap();

				find_literal_lexical(rdf, r, XSD_STRING)
					.map(Cow::into_owned)
					.map(Value::string)
			}
			Self::TypedStringOf => {
				let ty = args[0].as_resource().unwrap();
				let r = args[1].as_resource().unwrap();

				find_literal_interpreted(rdf, r, ty)
					.map(Cow::into_owned)
					.map(Value::string)
			}
		}
	}

	pub fn eval_inverse<P, R>(
		&self,
		rdf: &mut impl EvalContext<Resource = R>,
		scope: &mut Scope<P, R>,
		output: &Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>>
	where
		R: Clone,
	{
		match self {
			Self::Get => {
				let predicate = rdf.new_resource();
				let subject = rdf.new_resource();

				rdf.insert(Quad(
					subject.clone(),
					predicate.clone(),
					output.as_resource().unwrap().clone(),
					None,
				));

				Ok(vec![Value::Resource(predicate), Value::Resource(subject)])
			}
			Self::IriOf => {
				let value = output.as_str().unwrap();
				let iri = Iri::new(value).map_err(|_| EvalError::InvalidValue)?;
				let r = rdf.insert_iri(Cow::Borrowed(iri));
				Ok(vec![Value::Resource(r)])
			}
			Self::BoolOf => {
				let value = xsd_types::Boolean(output.as_bool().unwrap());
				let r = rdf.insert_literal(CowLiteral::new(value.to_string(), XSD_BOOLEAN));
				Ok(vec![Value::Resource(r)])
			}
			Self::TypedBoolOf => {
				let ty = iri_of(rdf, scope.get(0).unwrap().as_resource().unwrap())?.into_owned();

				let value = xsd_types::Boolean(output.as_bool().unwrap());
				let r = rdf.insert_literal(CowLiteral::new(value.to_string(), ty));
				Ok(vec![Value::Resource(r)])
			}
			Self::NumberOf => {
				let value: xsd_types::Decimal = output
					.as_number()
					.unwrap()
					.clone()
					.try_into()
					.map_err(|_| EvalError::InvalidValue)?;
				let r = rdf.insert_literal(CowLiteral::new(value.to_string(), XSD_DECIMAL));
				Ok(vec![Value::Resource(r)])
			}
			Self::TypedNumberOf => {
				let ty = iri_of(rdf, scope.get(0).unwrap().as_resource().unwrap())?.into_owned();

				let value: xsd_types::Decimal = output
					.as_number()
					.unwrap()
					.clone()
					.try_into()
					.map_err(|_| EvalError::InvalidValue)?;
				let r = rdf.insert_literal(CowLiteral::new(value.to_string(), ty));
				Ok(vec![Value::Resource(r)])
			}
			Self::IntOf => {
				let value: xsd_types::Integer = output
					.as_number()
					.unwrap()
					.clone()
					.try_into()
					.map_err(|_| EvalError::InvalidValue)?;
				let r = rdf.insert_literal(CowLiteral::new(value.to_string(), XSD_INTEGER));
				Ok(vec![Value::Resource(r)])
			}
			Self::TypedIntOf => {
				let ty = iri_of(rdf, scope.get(0).unwrap().as_resource().unwrap())?.into_owned();

				let value: xsd_types::Integer = output
					.as_number()
					.unwrap()
					.clone()
					.try_into()
					.map_err(|_| EvalError::InvalidValue)?;
				let r = rdf.insert_literal(CowLiteral::new(value.to_string(), ty));
				Ok(vec![Value::Resource(r)])
			}
			Self::UintOf => {
				let value: xsd_types::NonNegativeInteger = output
					.as_number()
					.unwrap()
					.clone()
					.try_into()
					.map_err(|_| EvalError::InvalidValue)?;
				let r = rdf
					.insert_literal(CowLiteral::new(value.to_string(), XSD_NON_NEGATIVE_INTEGER));
				Ok(vec![Value::Resource(r)])
			}
			Self::TypedUintOf => {
				let ty = iri_of(rdf, scope.get(0).unwrap().as_resource().unwrap())?.into_owned();

				let value: xsd_types::NonNegativeInteger = output
					.as_number()
					.unwrap()
					.clone()
					.try_into()
					.map_err(|_| EvalError::InvalidValue)?;
				let r = rdf.insert_literal(CowLiteral::new(value.to_string(), ty));
				Ok(vec![Value::Resource(r)])
			}
			Self::StringOf => {
				let value = output.as_str().unwrap();
				let r = rdf.insert_literal(CowLiteral::new(value.to_owned(), XSD_STRING));
				Ok(vec![Value::Resource(r)])
			}
			Self::TypedStringOf => {
				let ty = iri_of(rdf, scope.get(0).unwrap().as_resource().unwrap())?.into_owned();

				let value = output.as_str().unwrap();
				let r = rdf.insert_literal(CowLiteral::new(value.to_string(), ty));
				Ok(vec![Value::Resource(r)])
			}
		}
	}
}

fn iri_of<'a, R>(
	rdf: &'a impl EvalContext<Resource = R>,
	resource: &'a R,
) -> Result<Cow<'a, Iri>, EvalError<R>> {
	let mut iris = rdf.iris_of(resource);

	match iris.next() {
		Some(i) => {
			if iris.next().is_some() {
				Err(EvalError::Ambiguity)
			} else {
				Ok(i)
			}
		}
		None => Err(EvalError::MissingData(
			MissingData::NoMatchingIriRepresentation,
		)),
	}
}

fn find_literal_lexical<'a, R: PartialEq>(
	rdf: &'a impl EvalContext<Resource = R>,
	resource: &'a R,
	expected_ty: &Iri,
) -> Result<Cow<'a, str>, EvalError<R>> {
	let mut value = None;
	for literal in rdf.literals_of(resource) {
		let type_resource = match literal.type_ {
			CowLiteralType::Any(i) => i,
			CowLiteralType::LangString(_) => Cow::Borrowed(RDF_LANG_STRING),
		};

		if type_resource.as_ref() == expected_ty {
			if let Some(old_value) = value.replace(literal.value) {
				return Err(EvalError::AmbiguousLiteralRepresentation(
					old_value.into_owned(),
					value.unwrap().into_owned(),
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
	rdf: &'a impl EvalContext<Resource = R>,
	resource: &'a R,
	expected_ty: &R,
) -> Result<Cow<'a, str>, EvalError<R>> {
	let mut value = None;
	for literal in rdf.literals_of(resource) {
		let type_resource = match literal.type_ {
			CowLiteralType::Any(i) => rdf.iri_interpretation(&i),
			CowLiteralType::LangString(_) => rdf.iri_interpretation(RDF_LANG_STRING),
		};

		if let Some(type_resource) = type_resource {
			if type_resource == *expected_ty {
				if let Some(old_value) = value.replace(literal.value) {
					return Err(EvalError::AmbiguousLiteralRepresentation(
						old_value.into_owned(),
						value.unwrap().into_owned(),
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
