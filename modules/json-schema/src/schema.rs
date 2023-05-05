use iref::{IriBuf, IriRefBuf};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod validation;
pub use validation::*;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
	One(T),
	Many(Vec<T>),
}

impl<T> OneOrMany<T> {
	pub fn into_vec(self) -> Vec<T> {
		match self {
			Self::One(t) => vec![t],
			Self::Many(v) => v,
		}
	}

	pub fn as_slice(&self) -> &[T] {
		match self {
			Self::One(t) => std::slice::from_ref(t),
			Self::Many(v) => v,
		}
	}
}

// pub mod from_syntax;
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Schema {
	Boolean(bool),
	Ref(RefSchema),
	DynamicRef(DynamicRefSchema),
	Regular(RegularSchema),
}

impl Schema {
	pub fn as_ref(&self) -> Option<&RefSchema> {
		match self {
			Self::Ref(r) => Some(r),
			_ => None,
		}
	}

	pub fn as_dynamic_ref(&self) -> Option<&DynamicRefSchema> {
		match self {
			Self::DynamicRef(r) => Some(r),
			_ => None,
		}
	}

	pub fn as_regular(&self) -> Option<&RegularSchema> {
		match self {
			Self::Regular(r) => Some(r),
			_ => None,
		}
	}

	pub fn meta_data(&self) -> Option<&MetaData> {
		match self {
			Self::Regular(r) => Some(&r.meta_data),
			_ => None,
		}
	}
}

impl From<RefSchema> for Schema {
	fn from(s: RefSchema) -> Self {
		Self::Ref(s)
	}
}

impl From<DynamicRefSchema> for Schema {
	fn from(s: DynamicRefSchema) -> Self {
		Self::DynamicRef(s)
	}
}

impl From<RegularSchema> for Schema {
	fn from(s: RegularSchema) -> Self {
		Self::Regular(s)
	}
}

/// Regular schema definition.
#[derive(Serialize, Deserialize)]
pub struct RegularSchema {
	/// Meta schema properties.
	#[serde(flatten)]
	pub meta_schema: MetaSchema,

	/// Schema identifier.
	#[serde(rename = "$id")]
	pub id: Option<IriBuf>,

	/// Meta data.
	#[serde(flatten)]
	pub meta_data: MetaData,

	/// Schema description.
	#[serde(flatten)]
	pub desc: Description,

	/// Schema validation.
	#[serde(flatten)]
	pub validation: Validation,

	#[serde(rename = "$anchor")]
	pub anchor: Option<String>,

	#[serde(rename = "$dynamicAnchor")]
	pub dynamic_anchor: Option<String>,

	/// The "$defs" keyword reserves a location for schema authors to inline
	/// re-usable JSON Schemas into a more general schema. The keyword does not
	/// directly affect the validation result.
	#[serde(rename = "$defs")]
	pub defs: Option<BTreeMap<String, Schema>>,
}

impl RegularSchema {
	pub fn is_primitive(&self) -> bool {
		self.desc.is_empty() && self.validation.is_primitive()
	}
}

/// A Vocabulary for Basic Meta-Data Annotations.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
	pub title: Option<String>,
	pub description: Option<String>,
	pub default: Option<json_syntax::Value>,
	pub deprecated: Option<bool>,
	pub read_only: Option<bool>,
	pub write_only: Option<bool>,
	pub examples: Option<Vec<json_syntax::Value>>,
}

impl MetaData {
	pub fn is_empty(&self) -> bool {
		self.title.is_none()
			&& self.description.is_none()
			&& self.default.is_none()
			&& self.deprecated.is_none()
			&& self.read_only.is_none()
			&& self.write_only.is_none()
			&& self.examples.is_none()
	}
}

/// Meta-Schemas and Vocabularies.
#[derive(Serialize, Deserialize)]
pub struct MetaSchema {
	/// The "$schema" keyword is both used as a JSON Schema dialect identifier
	/// and as the identifier of a resource which is itself a JSON Schema, which
	/// describes the set of valid schemas written for this particular dialect.
	#[serde(rename = "$schema")]
	pub schema: Option<IriBuf>,

	/// The "$vocabulary" keyword is used in meta-schemas to identify the
	/// vocabularies available for use in schemas described by that meta-schema.
	/// It is also used to indicate whether each vocabulary is required or
	/// optional, in the sense that an implementation MUST understand the
	/// required vocabularies in order to successfully process the schema.
	/// Together, this information forms a dialect. Any vocabulary that is
	/// understood by the implementation MUST be processed in a manner
	/// consistent with the semantic definitions contained within the
	/// vocabulary.
	#[serde(rename = "$vocabulary")]
	pub vocabulary: Option<BTreeMap<IriBuf, bool>>,
}

impl MetaSchema {
	pub fn is_empty(&self) -> bool {
		self.schema.is_none() && self.vocabulary.is_none()
	}
}

/// Schema defined with the `$ref` keyword.
#[derive(Serialize, Deserialize)]
pub struct RefSchema {
	#[serde(rename = "$ref")]
	pub target: IriRefBuf,

	#[serde(flatten)]
	pub meta_data: MetaData,
}

/// Schema defined with the `$dynamicRef` keyword.
#[derive(Serialize, Deserialize)]
pub struct DynamicRefSchema {
	#[serde(rename = "$ref")]
	pub target: IriRefBuf,

	#[serde(flatten)]
	pub meta_data: MetaData,
}

/// Schema description.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Description {
	AllOf(AllOf),
	AnyOf(AnyOf),
	OneOf(OneOf),
	Not(Not),
	If(IfThenElse),
	Definition(Definition),
}

#[derive(Serialize, Deserialize)]
pub struct AllOf {
	#[serde(rename = "allOf")]
	pub schemas: Vec<Schema>,
}

#[derive(Serialize, Deserialize)]
pub struct AnyOf {
	#[serde(rename = "anyOf")]
	pub schemas: Vec<Schema>,
}

#[derive(Serialize, Deserialize)]
pub struct OneOf {
	#[serde(rename = "oneOf")]
	pub schemas: Vec<Schema>,
}

#[derive(Serialize, Deserialize)]
pub struct Not {
	#[serde(rename = "not")]
	pub schema: Box<Schema>,
}

#[derive(Serialize, Deserialize)]
pub struct IfThenElse {
	#[serde(rename = "if")]
	pub condition: Box<Schema>,

	pub then: Option<Box<Schema>>,

	#[serde(rename = "else")]
	pub els: Option<Box<Schema>>,
}

#[derive(Serialize, Deserialize)]
pub struct Definition {
	#[serde(flatten)]
	pub string: StringEncodedData,

	#[serde(flatten)]
	pub array: ArraySchema,

	#[serde(flatten)]
	pub object: ObjectSchema,
}

impl Definition {
	pub fn is_empty(&self) -> bool {
		self.string.is_empty() && self.array.is_empty() && self.object.is_empty()
	}
}

impl Description {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::Definition(def) => def.is_empty(),
			_ => false,
		}
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArraySchema {
	/// Validation succeeds if each element of the instance validates against
	/// the schema at the same position, if any. This keyword does not constrain
	/// the length of the array. If the array is longer than this keyword's
	/// value, this keyword validates only the prefix of matching length.
	///
	/// Omitting this keyword has the same assertion behavior as an empty array.
	pub prefix_items: Option<Vec<Schema>>,

	/// This keyword applies its subschema to all instance elements at indexes
	/// greater than the length of the "prefixItems" array in the same schema
	/// object, as reported by the annotation result of that "prefixItems"
	/// keyword. If no such annotation result exists, "items" applies its
	/// subschema to all instance array elements. [CREF11]
	///
	/// If the "items" subschema is applied to any positions within the instance
	/// array, it produces an annotation result of boolean true, indicating that
	/// all remaining array elements have been evaluated against this keyword's
	/// subschema.
	///
	/// Omitting this keyword has the same assertion behavior as an empty
	/// schema.
	pub items: Option<Box<Schema>>,

	/// An array instance is valid against "contains" if at least one of its
	/// elements is valid against the given schema. The subschema MUST be
	/// applied to every array element even after the first match has been
	/// found, in order to collect annotations for use by other keywords. This
	/// is to ensure that all possible annotations are collected.
	pub contains: Option<Box<Schema>>,

	/// The behavior of this keyword depends on the annotation results of
	/// adjacent keywords that apply to the instance location being validated.
	/// Specifically, the annotations from "prefixItems", "items", and
	/// "contains", which can come from those keywords when they are adjacent to
	/// the "unevaluatedItems" keyword. Those three annotations, as well as
	/// "unevaluatedItems", can also result from any and all adjacent in-place
	/// applicator keywords. This includes but is not limited to the in-place
	/// applicators defined in this document.
	pub unevaluated_items: Option<Box<Schema>>,
}

impl ArraySchema {
	pub fn is_empty(&self) -> bool {
		self.prefix_items.is_none()
			&& self.items.is_none()
			&& self.contains.is_none()
			&& self.unevaluated_items.is_none()
	}
}

/// Keywords for Applying Subschemas to Objects.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectSchema {
	/// Validation succeeds if, for each name that appears in both the instance
	/// and as a name within this keyword's value, the child instance for that
	/// name successfully validates against the corresponding schema.
	/// The annotation result of this keyword is the set of instance property
	/// names matched by this keyword.
	/// Omitting this keyword has the same assertion behavior as an empty
	/// object.
	pub properties: Option<BTreeMap<String, Schema>>,

	/// The value of "patternProperties" MUST be an object.
	/// Each property name of this object SHOULD be a valid regular expression,
	/// according to the ECMA-262 regular expression dialect.
	/// Each property value of this object MUST be a valid JSON Schema.
	pub pattern_properties: Option<BTreeMap<String, Schema>>,

	/// The behavior of this keyword depends on the presence and annotation
	/// results of "properties" and "patternProperties" within the same schema
	/// object. Validation with "additionalProperties" applies only to the child
	/// values of instance names that do not appear in the annotation results of
	/// either "properties" or "patternProperties".
	pub additional_properties: Option<Box<Schema>>,

	/// This keyword specifies subschemas that are evaluated if the instance is
	/// an object and contains a certain property.
	///
	/// This keyword's value MUST be an object. Each value in the object MUST be
	/// a valid JSON Schema.
	///
	/// If the object key is a property in the instance, the entire instance
	/// must validate against the subschema. Its use is dependent on the
	/// presence of the property.
	///
	/// Omitting this keyword has the same behavior as an empty object.
	pub dependent_schemas: Option<BTreeMap<String, Schema>>,

	/// The behavior of this keyword depends on the annotation results of
	/// adjacent keywords that apply to the instance location being validated.
	/// Specifically, the annotations from "properties", "patternProperties",
	/// and "additionalProperties", which can come from those keywords when they
	/// are adjacent to the "unevaluatedProperties" keyword. Those three
	/// annotations, as well as "unevaluatedProperties", can also result from
	/// any and all adjacent in-place applicator keywords. This includes but is
	/// not limited to the in-place applicators defined in this document.
	///
	/// Validation with "unevaluatedProperties" applies only to the child values
	/// of instance names that do not appear in the "properties",
	/// "patternProperties", "additionalProperties", or "unevaluatedProperties"
	/// annotation results that apply to the instance location being validated.
	///
	/// For all such properties, validation succeeds if the child instance
	/// validates against the "unevaluatedProperties" schema.
	///
	/// This means that "properties", "patternProperties",
	/// "additionalProperties", and all in-place applicators MUST be evaluated
	/// before this keyword can be evaluated. Authors of extension keywords MUST
	/// NOT define an in-place applicator that would need to be evaluated after
	/// this keyword.
	pub unevaluated_properties: Option<Box<Schema>>,
}

impl ObjectSchema {
	pub fn is_empty(&self) -> bool {
		self.properties.is_none()
			&& self.pattern_properties.is_none()
			&& self.additional_properties.is_none()
			&& self.dependent_schemas.is_none()
			&& self.unevaluated_properties.is_none()
	}
}

/// A Vocabulary for the Contents of String-Encoded Data
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StringEncodedData {
	/// Defines that the string SHOULD be interpreted as binary data and decoded
	/// using the encoding named by this property.
	pub content_encoding: Option<String>,

	/// If the instance is a string, this property indicates the media type of
	/// the contents of the string. If "contentEncoding" is present, this
	/// property describes the decoded string.
	///
	/// The value of this property MUST be a string, which MUST be a media type,
	/// as defined by RFC 2046.
	pub content_media_type: Option<String>,

	/// If the instance is a string, and if "contentMediaType" is present, this
	/// property contains a schema which describes the structure of the string.
	///
	/// This keyword MAY be used with any media type that can be mapped into
	/// JSON Schema's data model.
	///
	/// The value of this property MUST be a valid JSON schema. It SHOULD be
	/// ignored if "contentMediaType" is not present.
	pub content_schema: Option<Box<Schema>>,
}

impl StringEncodedData {
	pub fn is_empty(&self) -> bool {
		self.content_encoding.is_none()
			&& self.content_media_type.is_none()
			&& self.content_schema.is_none()
	}
}
