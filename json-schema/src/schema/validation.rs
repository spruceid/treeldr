use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Null,
	Boolean,
	Number,
	Integer,
	String,
	Array,
	Object,
}

pub struct Validation {
	pub any: AnyValidation,
	pub numeric: NumericValidation,
	pub string: StringValidation,
	pub array: ArrayValidation,
	pub object: ObjectValidation,
	pub format: Option<Format>,
}

impl Validation {
	/// Returns `true` is the only values set are the `type` and `format` properties.
	pub fn is_primitive(&self) -> bool {
		self.any.cnst.is_none()
			&& self.any.enm.is_none()
			&& self.numeric.is_empty()
			&& self.string.is_empty()
			&& self.array.is_empty()
	}
}

/// Validation Keywords for Any Instance Type.
pub struct AnyValidation {
	pub ty: Option<Vec<Type>>,
	pub enm: Option<Vec<serde_json::Value>>,
	pub cnst: Option<serde_json::Value>,
}

/// Validation Keywords for Numeric Instances (number and integer).
pub struct NumericValidation {
	/// The value of "multipleOf" MUST be a number, strictly greater than 0.
	///
	/// A numeric instance is valid only if division by this keyword's value
	/// results in an integer.
	pub multiple_of: Option<serde_json::Number>,

	/// The value of "maximum" MUST be a number, representing an inclusive upper
	/// limit for a numeric instance.
	///
	/// If the instance is a number, then this keyword validates only if the
	/// instance is less than or exactly equal to "maximum".
	pub maximum: Option<serde_json::Number>,

	/// The value of "exclusiveMaximum" MUST be a number, representing an
	/// exclusive upper limit for a numeric instance.
	///
	/// If the instance is a number, then the instance is valid only if it has a
	/// value strictly less than (not equal to) "exclusiveMaximum".
	pub exclusive_maximum: Option<serde_json::Number>,

	/// The value of "minimum" MUST be a number, representing an inclusive lower
	/// limit for a numeric instance.
	///
	/// If the instance is a number, then this keyword validates only if the
	/// instance is greater than or exactly equal to "minimum".
	pub minimum: Option<serde_json::Number>,

	/// The value of "exclusiveMinimum" MUST be a number, representing an
	/// exclusive lower limit for a numeric instance.
	///
	/// If the instance is a number, then the instance is valid only if it has a
	/// value strictly greater than (not equal to) "exclusiveMinimum".
	pub exclusive_minimum: Option<serde_json::Number>,
}

impl NumericValidation {
	pub fn is_empty(&self) -> bool {
		self.multiple_of.is_none()
			&& self.maximum.is_none()
			&& self.exclusive_maximum.is_none()
			&& self.minimum.is_none()
			&& self.exclusive_minimum.is_none()
	}
}

/// Validation Keywords for Strings
pub struct StringValidation {
	/// A string instance is valid against this keyword if its length is less
	/// than, or equal to, the value of this keyword.
	///
	/// The length of a string instance is defined as the number of its
	/// characters as defined by RFC 8259.
	pub max_length: Option<u64>,

	/// A string instance is valid against this keyword if its length is greater
	/// than, or equal to, the value of this keyword.
	///
	/// The length of a string instance is defined as the number of its
	/// characters as defined by RFC 8259.
	///
	/// Omitting this keyword has the same behavior as a value of 0.
	pub min_length: Option<u64>,

	/// The value of this keyword MUST be a string. This string SHOULD be a
	/// valid regular expression, according to the ECMA-262 regular expression
	/// dialect.
	///
	/// A string instance is considered valid if the regular expression matches
	/// the instance successfully. Recall: regular expressions are not
	/// implicitly anchored.
	pub pattern: Option<String>,
}

impl StringValidation {
	pub fn is_empty(&self) -> bool {
		self.max_length.is_none() && self.min_length.is_none() && self.pattern.is_none()
	}
}

/// Validation Keywords for Arrays
pub struct ArrayValidation {
	/// The value of this keyword MUST be a non-negative integer.
	///
	/// An array instance is valid against "maxItems" if its size is less than,
	/// or equal to, the value of this keyword.
	pub max_items: Option<u64>,

	/// An array instance is valid against "minItems" if its size is greater
	/// than, or equal to, the value of this keyword.
	///
	/// Omitting this keyword has the same behavior as a value of 0.
	pub min_items: Option<u64>,

	/// If this keyword has boolean value false, the instance validates
	/// successfully. If it has boolean value true, the instance validates
	/// successfully if all of its elements are unique.
	///
	/// Omitting this keyword has the same behavior as a value of false.
	pub unique_items: Option<bool>,

	/// If "contains" is not present within the same schema object, then this
	/// keyword has no effect.
	///
	/// An instance array is valid against "maxContains" in two ways, depending
	/// on the form of the annotation result of an adjacent "contains" keyword.
	/// The first way is if the annotation result is an array and the length of
	/// that array is less than or equal to the "maxContains" value. The second
	/// way is if the annotation result is a boolean "true" and the instance
	/// array length is less than or equal to the "maxContains" value.
	pub max_contains: Option<u64>,

	/// If "contains" is not present within the same schema object, then this
	/// keyword has no effect.
	///
	/// An instance array is valid against "minContains" in two ways, depending
	/// on the form of the annotation result of an adjacent "contains" keyword.
	/// The first way is if the annotation result is an array and the length of
	/// that array is greater than or equal to the "minContains" value. The
	/// second way is if the annotation result is a boolean "true" and the
	/// instance array length is greater than or equal to the "minContains"
	/// value.
	///
	/// A value of 0 is allowed, but is only useful for setting a range of
	/// occurrences from 0 to the value of "maxContains". A value of 0 with no
	/// "maxContains" causes "contains" to always pass validation.
	///
	/// Omitting this keyword has the same behavior as a value of 1.
	pub min_contains: Option<u64>,
}

impl ArrayValidation {
	pub fn is_empty(&self) -> bool {
		self.max_items.is_none()
			&& self.min_items.is_none()
			&& self.unique_items.is_none()
			&& self.max_contains.is_none()
			&& self.min_contains.is_none()
	}
}

/// Validation Keywords for Objects
pub struct ObjectValidation {
	/// An object instance is valid against "maxProperties" if its number of
	/// properties is less than, or equal to, the value of this keyword.
	pub max_properties: Option<u64>,

	/// An object instance is valid against "minProperties" if its number of
	/// properties is greater than, or equal to, the value of this keyword.
	///
	/// Omitting this keyword has the same behavior as a value of 0.
	pub min_properties: Option<u64>,

	/// Elements of this array, if any, MUST be strings, and MUST be unique.
	///
	/// An object instance is valid against this keyword if every item in the
	/// array is the name of a property in the instance.
	///
	/// Omitting this keyword has the same behavior as an empty array.
	pub required: Option<Vec<String>>,

	/// Elements in each array, if any, MUST be strings, and MUST be unique.
	///
	/// This keyword specifies properties that are required if a specific other
	/// property is present. Their requirement is dependent on the presence of
	/// the other property.
	///
	/// Validation succeeds if, for each name that appears in both the instance
	/// and as a name within this keyword's value, every item in the
	/// corresponding array is also the name of a property in the instance.
	///
	/// Omitting this keyword has the same behavior as an empty object.
	pub dependent_required: Option<BTreeMap<String, Vec<String>>>,
}

impl ObjectValidation {
	pub fn is_empty(&self) -> bool {
		self.max_properties.is_none()
			&& self.min_properties.is_none()
			&& self.required.is_none()
			&& self.dependent_required.is_none()
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Format {
	/// A string instance is valid against this attribute if it is a valid
	/// representation according to the "date-time" production.
	DateTime,

	/// A string instance is valid against this attribute if it is a valid
	/// representation according to the "full-date" production.
	Date,

	/// A string instance is valid against this attribute if it is a valid
	/// representation according to the "full-time" production.
	Time,

	/// A string instance is valid against this attribute if it is a valid
	/// representation according to the "duration" production.
	Duration,

	/// As defined by the "Mailbox" ABNF rule in RFC 5321, section 4.1.2.
	Email,

	/// As defined by the extended "Mailbox" ABNF rule in RFC 6531, section 3.3.
	IdnEmail,

	/// As defined by RFC 1123, section 2.1, including host names produced using
	/// the Punycode algorithm specified in RFC 5891, section 4.4.
	Hostname,

	/// As defined by either RFC 1123 as for hostname, or an internationalized
	/// hostname as defined by RFC 5890, section 2.3.2.3.
	IdnHostname,

	/// An IPv4 address according to the "dotted-quad" ABNF syntax as defined in
	/// RFC 2673, section 3.2.
	Ipv4,

	/// An IPv6 address as defined in RFC 4291, section 2.2.
	Ipv6,

	/// A string instance is valid against this attribute if it is a valid URI,
	/// according to [RFC3986].
	Uri,

	/// A string instance is valid against this attribute if it is a valid URI
	/// Reference (either a URI or a relative-reference), according to
	/// [RFC3986].
	UriReference,

	/// A string instance is valid against this attribute if it is a valid IRI,
	/// according to [RFC3987].
	Iri,

	/// A string instance is valid against this attribute if it is a valid IRI
	/// Reference (either an IRI or a relative-reference), according to
	/// [RFC3987].
	IriReference,

	/// A string instance is valid against this attribute if it is a valid
	/// string representation of a UUID, according to [RFC4122].
	Uuid,

	/// A string instance is valid against this attribute if it is a valid URI
	/// Template (of any level), according to [RFC6570].
	///
	/// Note that URI Templates may be used for IRIs; there is no separate IRI
	/// Template specification.
	UriTemplate,

	/// A string instance is valid against this attribute if it is a valid JSON
	/// string representation of a JSON Pointer, according to RFC 6901, section
	/// 5.
	JsonPointer,

	/// A string instance is valid against this attribute if it is a valid
	/// Relative JSON Pointer.
	RelativeJsonPointer,

	/// A regular expression, which SHOULD be valid according to the ECMA-262
	/// regular expression dialect.
	///
	/// Implementations that validate formats MUST accept at least the subset of
	/// ECMA-262 defined in the Regular Expressions section of this
	/// specification, and SHOULD accept all valid ECMA-262 expressions.
	Regex,
}
