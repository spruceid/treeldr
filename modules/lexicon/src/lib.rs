pub mod export;
pub mod import;

use std::collections::BTreeMap;

use import::IntoTriples;
use rdf_types::Vocabulary;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Version {
	One = 1,
}

/// A lexicon document.
#[derive(Debug, Serialize, Deserialize)]
pub struct LexiconDoc {
	pub lexicon: Version,
	pub id: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub revision: Option<u32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	#[serde(rename = "defs")]
	pub definitions: Definitions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Definitions {
	#[serde(skip_serializing_if = "Option::is_none")]
	main: Option<LexMainUserType>,

	#[serde(flatten)]
	other: BTreeMap<String, LexUserType>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum LexMainUserType {
	Record(LexRecord),
	Query(LexXrpcQuery),
	Procedure(LexXrpcProcedure),
	Subscription(LexXrpcSubscription),
}

impl From<LexMainUserType> for LexUserType {
	fn from(value: LexMainUserType) -> Self {
		match value {
			LexMainUserType::Record(r) => Self::Record(r),
			LexMainUserType::Query(q) => Self::Query(q),
			LexMainUserType::Procedure(p) => Self::Procedure(p),
			LexMainUserType::Subscription(s) => Self::Subscription(s),
		}
	}
}

impl LexiconDoc {
	pub fn into_triples<V: Vocabulary, G>(
		self,
		vocabulary: &mut V,
		generator: G,
	) -> IntoTriples<V, G> {
		IntoTriples::new(self, vocabulary, generator)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexRef {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	#[serde(rename = "ref")]
	pub ref_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexRefUnion {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	pub refs: Vec<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub closed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum LexRefVariant {
	Ref(LexRef),
	Union(LexRefUnion),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum LexUserType {
	Record(LexRecord),
	Query(LexXrpcQuery),
	Procedure(LexXrpcProcedure),
	Subscription(LexXrpcSubscription),
	Array(LexArray),
	Token(LexToken),
	Object(LexObject),
	Boolean(LexBoolean),
	Integer(LexInteger),
	String(LexString),
	Bytes(LexBytes),
	CidLink(LexCidLink),
	Unknown(LexUnknown),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcQuery {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub parameters: Option<LexXrpcParameters>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub output: Option<LexXrpcBody>,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub errors: Vec<LexXrpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LexXrpcParametersType {
	Params,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcParameters {
	#[serde(rename = "type")]
	pub type_: LexXrpcParametersType,

	pub description: Option<String>,

	#[serde(default)]
	pub required: Vec<String>,

	pub properties: BTreeMap<String, LexXrpcParametersProperty>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LexXrpcParametersProperty {
	Primitive(LexPrimitive),
	NonPrimitive(LexXrpcParametersNonPrimitiveProperty),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LexXrpcParametersNonPrimitiveProperty {
	Array(LexPrimitiveArray),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcBody {
	pub description: Option<String>,

	pub encoding: LexXrpcBodyEncoding,

	pub schema: LexXrpcBodySchema,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LexXrpcBodyEncoding {
	One(String),
	Many(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LexXrpcBodySchema {
	Object(LexObject),
	Ref(LexRefVariant),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcSubscriptionMessage {
	pub description: Option<String>,
	pub schema: LexXrpcBodySchema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcError {
	pub name: String,
	pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcProcedure {
	pub description: Option<String>,

	#[serde(default)]
	pub parameters: BTreeMap<String, LexPrimitive>,

	pub input: Option<LexXrpcBody>,

	pub output: Option<LexXrpcBody>,

	#[serde(default)]
	pub errors: Vec<LexXrpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexRecord {
	pub description: Option<String>,
	pub key: Option<String>,
	pub record: LexObject,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcSubscription {
	pub description: Option<String>,
	pub parameters: Option<LexXrpcParameters>,
	pub message: Option<LexXrpcSubscriptionMessage>,

	#[serde(default)]
	pub infos: Vec<LexXrpcError>,

	#[serde(default)]
	pub errors: Vec<LexXrpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexToken {
	pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexObject {
	pub description: Option<String>,

	#[serde(default)]
	pub required: Vec<String>,

	#[serde(default)]
	pub nullable: Vec<String>,

	#[serde(default)]
	pub properties: BTreeMap<String, ObjectProperty>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObjectProperty {
	Ref(LexRefVariant),
	Ipld(LexIpldType),
	Primitive(LexPrimitive),
	NonPrimitive(ObjectNonPrimitiveProperty),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ObjectNonPrimitiveProperty {
	Array(LexArray),
	Blob(LexBlob),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LexIpldType {
	Bytes(LexBytes),
	CidLink(LexCidLink),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexCidLink {
	pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexBytes {
	pub description: Option<String>,

	pub min_size: Option<u32>,

	pub max_size: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexBlob {
	pub description: Option<String>,

	pub accept: Option<Vec<String>>,

	pub max_size: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexImage {
	pub description: Option<String>,
	pub accept: Option<Vec<String>>,
	pub max_size: Option<u32>,
	pub max_width: Option<u32>,
	pub max_height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexVideo {
	pub description: Option<String>,
	pub accept: Option<Vec<String>>,
	pub max_size: Option<u32>,
	pub max_width: Option<u32>,
	pub max_height: Option<u32>,
	pub max_length: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexAudio {
	pub description: Option<String>,
	pub accept: Option<Vec<String>>,
	pub max_size: Option<u32>,
	pub max_length: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexArray<T = ArrayItem> {
	pub description: Option<String>,
	pub items: T,
	pub min_length: Option<u32>,
	pub max_length: Option<u32>,
}

pub type LexPrimitiveArray = LexArray<LexPrimitive>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArrayItem {
	Primitive(LexPrimitive),
	Ipld(LexIpldType),
	Blob(LexBlob),
	Ref(LexRefVariant),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum LexPrimitive {
	Boolean(LexBoolean),
	Integer(LexInteger),
	String(LexString),
	Unknown(LexUnknown),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexBoolean {
	pub description: Option<String>,
	pub default: Option<bool>,
	pub const_: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexNumber {
	pub default: Option<f64>,
	pub minimum: Option<f64>,
	pub maximum: Option<f64>,
	pub enum_: Option<Vec<f64>>,
	pub const_: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexInteger {
	pub description: Option<String>,
	pub default: Option<i64>,
	pub minimum: Option<i64>,
	pub maximum: Option<i64>,
	pub enum_: Option<Vec<i64>>,
	pub const_: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexString {
	pub description: Option<String>,
	pub default: Option<String>,
	pub format: Option<LexStringFormat>,
	pub min_length: Option<u32>,
	pub max_length: Option<u32>,
	pub min_grapheme: Option<u32>,
	pub max_grapheme: Option<u32>,
	pub enum_: Option<Vec<String>>,
	pub const_: Option<String>,

	#[serde(default)]
	pub known_values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LexStringFormat {
	Datetime,
	Uri,
	AtUri,
	Did,
	Handle,
	AtIdentifier,
	Nsid,
	Cid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexUnknown {
	pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexParams {
	#[serde(default)]
	pub properties: BTreeMap<String, ObjectProperty>,
}
