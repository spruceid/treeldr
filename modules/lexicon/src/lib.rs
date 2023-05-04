pub mod import;
pub mod export;

use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Version {
	One = 1
}

/// A lexicon document.
#[derive(Serialize, Deserialize)]
pub struct LexiconDoc {
	lexicon: Version,
	id: String,
	revision: Option<u32>,
	description: Option<String>,

	#[serde(rename = "defs")]
	definitions: BTreeMap<String, Definition>
}

#[derive(Serialize, Deserialize)]
pub struct LexRef(String);

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Definition {
	UserType(LexUserType),
	Array(LexArray),
	Primitive(LexPrimitive),
	Ref(LexRef)
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum LexUserType {
	Query(LexXrpcQuery),
	Procedure(LexXrpcProcedure),
	Record(LexRecord),
	Token(LexToken),
	Object(LexObject),
	Blob(LexBlob),
	Image(LexImage),
	Video(LexVideo),
	Audio(LexAudio)
}

#[derive(Serialize, Deserialize)]
pub struct LexXrpcQuery {
	description: Option<String>,
	
	#[serde(default)]
	parameters: BTreeMap<String, LexPrimitive>,

	output: Option<LexXrpcBody>,
	
	#[serde(default)]
	errors: Vec<LexXrpcError>
}

#[derive(Serialize, Deserialize)]
pub struct LexXrpcBody {
	description: Option<String>,

	encoding: Encoding,

	schema: LexObject
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Encoding {
	One(String),
	Many(Vec<String>)
}

#[derive(Serialize, Deserialize)]
pub struct LexXrpcError {
	name: String,
	description: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct LexXrpcProcedure {
	description: Option<String>,
	
	#[serde(default)]
	parameters: BTreeMap<String, LexPrimitive>,

	input: Option<LexXrpcBody>,

	output: Option<LexXrpcBody>,
	
	#[serde(default)]
	errors: Vec<LexXrpcError>
}

#[derive(Serialize, Deserialize)]
pub struct LexRecord {
	description: Option<String>,
	key: Option<String>,
	record: LexObject
}

#[derive(Serialize, Deserialize)]
pub struct LexToken {
	description: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct LexObject {
	description: Option<String>,
	required: Vec<String>,

	#[serde(default)]
	properties: BTreeMap<String, Property>
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
	Ref(LexRef),
	Array(LexArray),
	Primitive(LexPrimitive),
	Refs(Vec<LexRef>)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexBlob {
	description: Option<String>,

	accept: Option<Vec<String>>,

	max_size: Option<u32>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexImage {
	description: Option<String>,
	accept: Option<Vec<String>>,
	max_size: Option<u32>,
	max_width: Option<u32>,
	max_height: Option<u32>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexVideo {
	description: Option<String>,
	accept: Option<Vec<String>>,
	max_size: Option<u32>,
	max_width: Option<u32>,
	max_height: Option<u32>,
	max_length: Option<u32>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexAudio {
	description: Option<String>,
	accept: Option<Vec<String>>,
	max_size: Option<u32>,
	max_length: Option<u32>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexArray {
	description: Option<String>,
	items: ArrayItem,
	min_length: Option<u32>,
	max_lenght: Option<u32>
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArrayItem {
	Ref(LexRef),
	Primitive(LexPrimitive),
	Refs(Vec<LexRef>)
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum LexPrimitive {
	Boolean(LexBoolean),
	Number(LexNumber),
	Integer(LexInteger),
	String(LexString)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexBoolean {
	default: Option<bool>,
	const_: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexNumber {
	default: Option<f64>,
	minimum: Option<f64>,
	maximum: Option<f64>,
	enum_: Option<Vec<f64>>,
	const_: Option<f64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexInteger {
	default: Option<i64>,
	minimum: Option<i64>,
	maximum: Option<i64>,
	enum_: Option<Vec<i64>>,
	const_: Option<i64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexString {
	default: Option<String>,
	min_length: Option<u32>,
	max_length: Option<u32>,
	enum_: Option<Vec<String>>,
	const_: Option<String>,

	#[serde(default)]
	known_values: Vec<String>
}