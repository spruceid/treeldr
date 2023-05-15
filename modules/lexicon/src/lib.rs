use std::collections::BTreeMap;

use import::IntoTriples;
use rdf_types::Vocabulary;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod export;
pub mod import;
mod nsid;

pub use nsid::*;

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Version {
	One = 1,
}

/// A lexicon document.
#[derive(Debug, Serialize, Deserialize)]
pub struct LexiconDoc {
	pub lexicon: Version,
	pub id: NsidBuf,

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
	main: Option<LexAnyUserType>,

	#[serde(flatten)]
	other: BTreeMap<String, LexUserType>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum LexAnyUserType {
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

impl From<LexUserType> for LexAnyUserType {
	fn from(value: LexUserType) -> Self {
		match value {
			LexUserType::Array(a) => Self::Array(a),
			LexUserType::Token(t) => Self::Token(t),
			LexUserType::Object(o) => Self::Object(o),
			LexUserType::Boolean(b) => Self::Boolean(b),
			LexUserType::Integer(i) => Self::Integer(i),
			LexUserType::String(s) => Self::String(s),
			LexUserType::Bytes(b) => Self::Bytes(b),
			LexUserType::CidLink(l) => Self::CidLink(l),
			LexUserType::Unknown(u) => Self::Unknown(u),
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

impl LexXrpcParametersProperty {
	pub fn description(&self) -> Option<&str> {
		match self {
			Self::Primitive(p) => p.description(),
			Self::NonPrimitive(n) => n.description(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum LexXrpcParametersNonPrimitiveProperty {
	Array(LexPrimitiveArray),
}

impl LexXrpcParametersNonPrimitiveProperty {
	pub fn description(&self) -> Option<&str> {
		match self {
			Self::Array(a) => a.description.as_deref(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LexXrpcBody {
	pub description: Option<String>,

	pub encoding: LexXrpcBodyEncoding,

	pub schema: Option<LexXrpcBodySchema>,
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
#[serde(rename_all = "kebab-case")]
pub enum ObjectNonPrimitiveProperty {
	Array(LexArray),
	Blob(LexBlob),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
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

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct LexImage {
// 	pub description: Option<String>,
// 	pub accept: Option<Vec<String>>,
// 	pub max_size: Option<u32>,
// 	pub max_width: Option<u32>,
// 	pub max_height: Option<u32>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct LexVideo {
// 	pub description: Option<String>,
// 	pub accept: Option<Vec<String>>,
// 	pub max_size: Option<u32>,
// 	pub max_width: Option<u32>,
// 	pub max_height: Option<u32>,
// 	pub max_length: Option<u32>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct LexAudio {
// 	pub description: Option<String>,
// 	pub accept: Option<Vec<String>>,
// 	pub max_size: Option<u32>,
// 	pub max_length: Option<u32>,
// }

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
	Ref(LexRefVariant),
	NonPrimitive(ArrayNonPrimitiveItem),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum ArrayNonPrimitiveItem {
	Blob(LexBlob),
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

impl LexPrimitive {
	pub fn description(&self) -> Option<&str> {
		match self {
			Self::Boolean(b) => b.description.as_deref(),
			Self::Integer(i) => i.description.as_deref(),
			Self::String(s) => s.description.as_deref(),
			Self::Unknown(u) => u.description.as_deref(),
		}
	}
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

impl LexInteger {
	/// Find the best fitting TreeLDR primitive integer layout for this integer
	/// type.
	pub fn best_primitive(&self) -> treeldr::vocab::Primitive {
		match (self.minimum, self.maximum) {
			(Some(min), Some(max)) if min >= u8::MIN as i64 && max <= u8::MAX as i64 => {
				treeldr::vocab::Primitive::U8
			}
			(Some(min), Some(max)) if min >= u16::MIN as i64 && max <= u16::MAX as i64 => {
				treeldr::vocab::Primitive::U16
			}
			(Some(min), Some(max)) if min >= u32::MIN as i64 && max <= u32::MAX as i64 => {
				treeldr::vocab::Primitive::U32
			}
			(Some(min), Some(_max)) if min >= u64::MIN as i64 => {
				// && max <= u8::MAX as i64
				treeldr::vocab::Primitive::U64
			}
			(Some(min), Some(max)) if min >= i8::MIN as i64 && max <= i8::MAX as i64 => {
				treeldr::vocab::Primitive::I8
			}
			(Some(min), Some(max)) if min >= i16::MIN as i64 && max <= i16::MAX as i64 => {
				treeldr::vocab::Primitive::I16
			}
			(Some(min), Some(max)) if min >= i32::MIN as i64 && max <= i32::MAX as i64 => {
				treeldr::vocab::Primitive::I32
			}
			(Some(_min), Some(_max)) => {
				// if min >= i64::MIN && max <= i64::MAX => {
				treeldr::vocab::Primitive::I64
			}
			(Some(min), _) if min > 0 => treeldr::vocab::Primitive::PositiveInteger,
			(Some(min), _) if min >= 0 => treeldr::vocab::Primitive::NonNegativeInteger,
			(_, Some(max)) if max < 0 => treeldr::vocab::Primitive::NegativeInteger,
			(_, Some(max)) if max <= 0 => treeldr::vocab::Primitive::NonPositiveInteger,
			_ => treeldr::vocab::Primitive::Integer,
		}
	}

	pub fn bounds_constraints(&self, p: treeldr::vocab::Primitive) -> (Option<i64>, Option<i64>) {
		match p {
			treeldr::vocab::Primitive::U8 => (
				self.minimum.filter(|m| *m > u8::MIN as i64),
				self.maximum.filter(|m| *m < u8::MAX as i64),
			),
			treeldr::vocab::Primitive::U16 => (
				self.minimum.filter(|m| *m > u16::MIN as i64),
				self.maximum.filter(|m| *m < u16::MAX as i64),
			),
			treeldr::vocab::Primitive::U32 => (
				self.minimum.filter(|m| *m > u32::MIN as i64),
				self.maximum.filter(|m| *m < u32::MAX as i64),
			),
			treeldr::vocab::Primitive::U64 => {
				(self.minimum.filter(|m| *m > u64::MIN as i64), self.maximum)
			}
			treeldr::vocab::Primitive::I8 => (
				self.minimum.filter(|m| *m > i8::MIN as i64),
				self.maximum.filter(|m| *m < i8::MAX as i64),
			),
			treeldr::vocab::Primitive::I16 => (
				self.minimum.filter(|m| *m > i16::MIN as i64),
				self.maximum.filter(|m| *m < i16::MAX as i64),
			),
			treeldr::vocab::Primitive::I32 => (
				self.minimum.filter(|m| *m > i32::MIN as i64),
				self.maximum.filter(|m| *m < i32::MAX as i64),
			),
			treeldr::vocab::Primitive::I64 => (
				self.minimum.filter(|m| *m > i64::MIN),
				self.maximum.filter(|m| *m < i64::MAX),
			),
			treeldr::vocab::Primitive::PositiveInteger => {
				(self.minimum.filter(|m| *m > 1), self.maximum)
			}
			treeldr::vocab::Primitive::NonNegativeInteger => {
				(self.minimum.filter(|m| *m > 0), self.maximum)
			}
			treeldr::vocab::Primitive::NegativeInteger => {
				(self.minimum, self.maximum.filter(|m| *m < -1))
			}
			treeldr::vocab::Primitive::NonPositiveInteger => {
				(self.minimum, self.maximum.filter(|m| *m < 0))
			}
			_ => (self.minimum, self.maximum),
		}
	}
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
