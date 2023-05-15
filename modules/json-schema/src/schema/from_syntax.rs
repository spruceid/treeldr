use super::*;
use iref::{IriBuf, IriRefBuf};
use json_syntax::Value;
use locspan::Meta;
use std::fmt;

#[derive(Debug)]
pub enum Error {
	InvalidSchema,
	InvalidUri,
	InvalidUriRef,
	InvalidType,
	NotABoolean,
	NotANumber,
	NotAPositiveInteger,
	NotAString,
	NotAnArray,
	NotAnObject,
	UnknownFormat,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidSchema => write!(f, "invalid `$schema` value"),
			Self::InvalidUri => write!(f, "invalid URI"),
			Self::InvalidUriRef => write!(f, "invalid URI reference"),
			Self::InvalidType => write!(f, "invalid `type` value"),
			Self::NotABoolean => write!(f, "expected a boolean"),
			Self::NotANumber => write!(f, "expected a number"),
			Self::NotAPositiveInteger => write!(f, "expected a positive integer"),
			Self::NotAString => write!(f, "expected a string"),
			Self::NotAnArray => write!(f, "expected an array"),
			Self::NotAnObject => write!(f, "expected an object"),
			Self::UnknownFormat => write!(f, "unknown `format` value"),
		}
	}
}

trait ValueTryInto: Sized {
	fn try_into_bool(self) -> Result<bool, Error>;
	fn try_into_number(self) -> Result<json_syntax::NumberBuf, Error>;
	fn try_into_u64(self) -> Result<u64, Error> {
		self.try_into_number()?
			.as_u64()
			.ok_or(Error::NotAPositiveInteger)
	}
	fn try_into_string(self) -> Result<String, Error>;
	fn try_into_array(self) -> Result<json_syntax::Array, Error>;

	fn try_into_schema_array(self) -> Result<Vec<Schema>, Error> {
		let mut schemas = Vec::new();
		for v in self.try_into_array()? {
			schemas.push(v.try_into_schema()?)
		}
		Ok(schemas)
	}

	fn try_into_string_array(self) -> Result<Vec<String>, Error> {
		let mut schemas = Vec::new();
		for v in self.try_into_array()? {
			schemas.push(v.try_into_string()?)
		}
		Ok(schemas)
	}

	fn try_into_schema(self) -> Result<Schema, Error>;

	fn try_into_boxed_schema(self) -> Result<Box<Schema>, Error> {
		Ok(Box::new(self.try_into_schema()?))
	}

	fn try_into_object(self) -> Result<json_syntax::Object, Error>;

	fn try_into_uri(self) -> Result<IriBuf, Error> {
		IriBuf::from_string(self.try_into_string()?).map_err(|_| Error::InvalidUri)
	}

	fn try_into_uri_ref(self) -> Result<IriRefBuf, Error> {
		IriRefBuf::from_string(self.try_into_string()?).map_err(|_| Error::InvalidUriRef)
	}
}

impl ValueTryInto for Value {
	fn try_into_bool(self) -> Result<bool, Error> {
		match self {
			Self::Boolean(b) => Ok(b),
			_ => Err(Error::NotABoolean),
		}
	}

	fn try_into_number(self) -> Result<json_syntax::NumberBuf, Error> {
		match self {
			Self::Number(n) => Ok(n),
			_ => Err(Error::NotANumber),
		}
	}

	fn try_into_string(self) -> Result<String, Error> {
		match self {
			Self::String(s) => Ok(s.into_string()),
			_ => Err(Error::NotAString),
		}
	}

	fn try_into_array(self) -> Result<json_syntax::Array, Error> {
		match self {
			Self::Array(a) => Ok(a),
			_ => Err(Error::NotAnArray),
		}
	}

	fn try_into_object(self) -> Result<json_syntax::Object, Error> {
		match self {
			Self::Object(o) => Ok(o),
			_ => Err(Error::NotAnObject),
		}
	}

	fn try_into_schema(self) -> Result<Schema, Error> {
		Schema::try_from(self)
	}
}

fn read_meta_data(value: &mut json_syntax::Object) -> Result<MetaData, Error> {
	Ok(MetaData {
		title: value
			.remove("title")
			.map(|t| t.try_into_string())
			.transpose()?,
		description: value
			.remove("description")
			.map(|t| t.try_into_string())
			.transpose()?,
		default: value.remove("default"),
		deprecated: value
			.remove("deprecated")
			.map(|t| t.try_into_bool())
			.transpose()?,
		read_only: value
			.remove("readOnly")
			.map(|t| t.try_into_bool())
			.transpose()?,
		write_only: value
			.remove("writeOnly")
			.map(|t| t.try_into_bool())
			.transpose()?,
		examples: value
			.remove("examples")
			.map(|t| t.try_into_array())
			.transpose()?,
	})
}

fn read_meta_schema(value: &mut json_syntax::Object) -> Result<MetaSchema, Error> {
	Ok(MetaSchema {
		schema: value
			.remove("$schema")
			.map(|t| t.try_into_uri())
			.transpose()?,
		vocabulary: value
			.remove("$vocabulary")
			.map(|t| {
				let obj = t.try_into_object()?;
				let mut vocab = BTreeMap::new();
				for (key, value) in obj {
					let uri = IriBuf::from_string(key).map_err(|_| Error::InvalidUriRef)?;
					vocab.insert(uri, value.try_into_bool()?);
				}
				Ok(vocab)
			})
			.transpose()?,
	})
}

fn read_description(value: &mut json_syntax::Object) -> Result<Description, Error> {
	if let Some(all_of) = value.remove("allOf") {
		Ok(Description::AllOf(all_of.try_into_schema_array()?))
	} else if let Some(any_of) = value.remove("anyOf") {
		Ok(Description::AnyOf(any_of.try_into_schema_array()?))
	} else if let Some(one_of) = value.remove("oneOf") {
		Ok(Description::OneOf(one_of.try_into_schema_array()?))
	} else if let Some(not) = value.remove("not") {
		Ok(Description::Not(not.try_into_boxed_schema()?))
	} else if let Some(condition) = value.remove("if") {
		Ok(Description::If {
			condition: Box::new(Schema::try_from(condition)?),
			then: value
				.remove("then")
				.map(|s| Ok(Box::new(s.try_into()?)))
				.transpose()?,
			els: value
				.remove("els")
				.map(|s| Ok(Box::new(s.try_into()?)))
				.transpose()?,
		})
	} else {
		Ok(Description::Definition {
			string: read_string_encoded_data_schema(value)?,
			array: read_array_schema(value)?,
			object: read_object_schema(value)?,
		})
	}
}

fn read_string_encoded_data_schema(
	value: &mut json_syntax::Object,
) -> Result<StringEncodedData, Error> {
	Ok(StringEncodedData {
		content_encoding: value
			.remove("contentEncoding")
			.map(ValueTryInto::try_into_string)
			.transpose()?,
		content_media_type: value
			.remove("contentMediaType")
			.map(ValueTryInto::try_into_string)
			.transpose()?,
		content_schema: value
			.remove("contentSchema")
			.map(|s| Ok(Box::new(s.try_into()?)))
			.transpose()?,
	})
}

fn read_array_schema(value: &mut json_syntax::Object) -> Result<ArraySchema, Error> {
	Ok(ArraySchema {
		prefix_items: value
			.remove("prefixItems")
			.map(ValueTryInto::try_into_schema_array)
			.transpose()?,
		items: value
			.remove("items")
			.map(ValueTryInto::try_into_boxed_schema)
			.transpose()?,
		contains: value
			.remove("contains")
			.map(ValueTryInto::try_into_boxed_schema)
			.transpose()?,
		unevaluated_items: value
			.remove("unevaluatedItems")
			.map(ValueTryInto::try_into_boxed_schema)
			.transpose()?,
	})
}

fn read_object_schema(value: &mut json_syntax::Object) -> Result<ObjectSchema, Error> {
	Ok(ObjectSchema {
		properties: value
			.remove("properties")
			.map(|v| {
				let obj = v.try_into_object()?;
				let mut properties = BTreeMap::new();
				for (key, value) in obj {
					properties.insert(key, value.try_into_schema()?);
				}
				Ok(properties)
			})
			.transpose()?,
		pattern_properties: value
			.remove("patternProperties")
			.map(|v| {
				let obj = v.try_into_object()?;
				let mut properties = BTreeMap::new();
				for (key, value) in obj {
					properties.insert(key, value.try_into_schema()?);
				}
				Ok(properties)
			})
			.transpose()?,
		additional_properties: value
			.remove("additionalProperties")
			.map(ValueTryInto::try_into_boxed_schema)
			.transpose()?,
		dependent_schemas: value
			.remove("dependentSchemas")
			.map(|v| {
				let obj = v.try_into_object()?;
				let mut properties = BTreeMap::new();
				for (key, value) in obj {
					properties.insert(key, value.try_into_schema()?);
				}
				Ok(properties)
			})
			.transpose()?,
		unevaluated_properties: value
			.remove("unevaluatedProperties")
			.map(ValueTryInto::try_into_boxed_schema)
			.transpose()?,
	})
}

fn read_validation(value: &mut json_syntax::Object) -> Result<Validation, Error> {
	Ok(Validation {
		any: read_any_validation(value)?,
		numeric: read_numeric_validation(value)?,
		string: read_string_validation(value)?,
		array: read_array_validation(value)?,
		object: read_object_validation(value)?,
		format: value.remove("format").map(Format::try_from).transpose()?,
	})
}

fn read_any_validation(value: &mut json_syntax::Object) -> Result<AnyValidation, Error> {
	Ok(AnyValidation {
		ty: value
			.remove("type")
			.map(|t| {
				Ok(match t {
					Value::Array(items) => {
						let mut types = Vec::with_capacity(items.len());
						for i in items {
							types.push(i.try_into()?);
						}
						types
					}
					t => vec![t.try_into()?],
				})
			})
			.transpose()?,
		enm: value
			.remove("enum")
			.map(ValueTryInto::try_into_array)
			.transpose()?,
		cnst: value.remove("const"),
	})
}

fn read_numeric_validation(
	value: &mut json_syntax::Object,
) -> Result<NumericValidation, Error> {
	Ok(NumericValidation {
		multiple_of: value
			.remove("multipleOf")
			.map(ValueTryInto::try_into_number)
			.transpose()?,
		maximum: value
			.remove("maximum")
			.map(ValueTryInto::try_into_number)
			.transpose()?,
		exclusive_maximum: value
			.remove("exclusiveMaximum")
			.map(ValueTryInto::try_into_number)
			.transpose()?,
		minimum: value
			.remove("minimum")
			.map(ValueTryInto::try_into_number)
			.transpose()?,
		exclusive_minimum: value
			.remove("exclusiveMinimum")
			.map(ValueTryInto::try_into_number)
			.transpose()?,
	})
}

fn read_string_validation(
	value: &mut json_syntax::Object,
) -> Result<StringValidation, Error> {
	Ok(StringValidation {
		max_length: value
			.remove("maxLength")
			.map(ValueTryInto::try_into_u64)
			.transpose()?,
		min_length: value
			.remove("minLength")
			.map(ValueTryInto::try_into_u64)
			.transpose()?,
		pattern: value
			.remove("pattern")
			.map(ValueTryInto::try_into_string)
			.transpose()?,
	})
}

fn read_array_validation(
	value: &mut json_syntax::Object,
) -> Result<ArrayValidation, Error> {
	Ok(ArrayValidation {
		max_items: value
			.remove("maxItems")
			.map(ValueTryInto::try_into_u64)
			.transpose()?,
		min_items: value
			.remove("minItems")
			.map(ValueTryInto::try_into_u64)
			.transpose()?,
		unique_items: value
			.remove("uniqueItems")
			.map(ValueTryInto::try_into_bool)
			.transpose()?,
		max_contains: value
			.remove("maxContains")
			.map(ValueTryInto::try_into_u64)
			.transpose()?,
		min_contains: value
			.remove("minContains")
			.map(ValueTryInto::try_into_u64)
			.transpose()?,
	})
}

fn read_object_validation(
	value: &mut json_syntax::Object,
) -> Result<ObjectValidation, Error> {
	Ok(ObjectValidation {
		max_properties: value
			.remove("maxProperties")
			.map(ValueTryInto::try_into_u64)
			.transpose()?,
		min_properties: value
			.remove("minProperties")
			.map(ValueTryInto::try_into_u64)
			.transpose()?,
		required: value
			.remove("required")
			.map(ValueTryInto::try_into_string_array)
			.transpose()?,
		dependent_required: value
			.remove("dependentRequired")
			.map(|v| {
				let obj = v.try_into_object()?;
				let mut map = BTreeMap::new();
				for (key, value) in obj {
					map.insert(key, value.try_into_string_array()?);
				}
				Ok(map)
			})
			.transpose()?,
	})
}

impl TryFrom<Value> for Type {
	type Error = Error;

	fn try_from(v: Value) -> Result<Self, Self::Error> {
		let s = v.try_into_string()?;
		let t = match s.as_str() {
			"null" => Self::Null,
			"boolean" => Self::Boolean,
			"number" => Self::Number,
			"integer" => Self::Integer,
			"string" => Self::String,
			"array" => Self::Array,
			"object" => Self::Object,
			_ => return Err(Error::InvalidType),
		};

		Ok(t)
	}
}

impl TryFrom<Value> for Format {
	type Error = Error;

	fn try_from(v: Value) -> Result<Self, Self::Error> {
		let s = v.try_into_string()?;
		let f = match s.as_str() {
			"date-time" => Self::DateTime,
			"date" => Self::Date,
			"time" => Self::Time,
			"duration" => Self::Duration,
			"email" => Self::Email,
			"idn-email" => Self::IdnEmail,
			"hostname" => Self::Hostname,
			"idn-hostname" => Self::IdnHostname,
			"ipv4" => Self::Ipv4,
			"ipv6" => Self::Ipv6,
			"uri" => Self::Uri,
			"uri-reference" => Self::UriReference,
			"iri" => Self::Iri,
			"iri-reference" => Self::IriReference,
			"uuid" => Self::Uuid,
			"uri-template" => Self::UriTemplate,
			"json-pointer" => Self::JsonPointer,
			"relative-json-pointer" => Self::RelativeJsonPointer,
			"regex" => Self::Regex,
			_ => return Err(Error::UnknownFormat),
		};

		Ok(f)
	}
}

impl TryFrom<Value> for Schema {
	type Error = Error;

	fn try_from(v: Value) -> Result<Self, Self::Error> {
		match v {
			Value::Boolean(true) => Ok(Self::True),
			Value::Boolean(false) => Ok(Self::False),
			Value::Object(mut obj) => {
				if let Some(value) = obj.remove("$ref") {
					let value = value.as_str().ok_or(Error::NotAString)?;
					let uri_ref = IriRefBuf::new(value).map_err(|_| Error::InvalidUriRef)?;
					Ok(Self::Ref(RefSchema {
						meta_data: read_meta_data(&mut obj)?,
						target: uri_ref,
					}))
				} else if let Some(value) = obj.remove("$dynamicRef") {
					let value = value.as_str().ok_or(Error::NotAString)?;
					let uri_ref = IriRefBuf::new(value).map_err(|_| Error::InvalidUriRef)?;
					Ok(Self::DynamicRef(DynamicRefSchema {
						meta_data: read_meta_data(&mut obj)?,
						target: uri_ref,
					}))
				} else {
					let meta_schema = read_meta_schema(&mut obj)?;
					let meta_data = read_meta_data(&mut obj)?;
					let id = obj
						.remove("$id")
						.map(ValueTryInto::try_into_uri)
						.transpose()
						.map_err(|_| Error::InvalidUri)?;
					let anchor = obj
						.remove("$anchor")
						.map(ValueTryInto::try_into_string)
						.transpose()?;
					let dynamic_anchor = obj
						.remove("$dynamicAnchor")
						.map(ValueTryInto::try_into_string)
						.transpose()?;
					let defs = obj
						.remove("$defs")
						.map(|t| {
							let obj = t.try_into_object()?;
							let mut defs = BTreeMap::new();
							for (key, value) in obj {
								let schema: Schema = value.try_into()?;
								defs.insert(key, schema);
							}
							Ok(defs)
						})
						.transpose()?;

					let desc = read_description(&mut obj)?;
					let validation = read_validation(&mut obj)?;

					Ok(Self::Regular(RegularSchema {
						meta_schema,
						id,
						meta_data,
						desc,
						validation,
						anchor,
						dynamic_anchor,
						defs,
					}))
				}
			}
			_ => Err(Error::InvalidSchema),
		}
	}
}
