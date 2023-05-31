use std::{borrow::Cow, fmt};

use iref::{AsIri, IriBuf};
use langtag::LanguageTag;
use locspan::Meta;
use rdf_types::IriVocabulary;
use treeldr_primitives::{BytesBuf, CidBuf, UriBuf, UrlBuf};
pub use xsd_types::value::*;
use xsd_types::ParseRdf;

use crate::{layout::primitive::RegExp, vocab, Id, IriIndex};

mod lang_string;
mod numeric;

pub use lang_string::LangString;
pub use numeric::*;

/// Value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
	Node(Id),
	Literal(Literal),
}

impl Value {
	pub fn as_id(&self) -> Option<Id> {
		match self {
			Self::Node(id) => Some(*id),
			Self::Literal(_) => None,
		}
	}

	pub fn into_id(self) -> Result<Id, Literal> {
		match self {
			Self::Node(id) => Ok(id),
			Self::Literal(l) => Err(l),
		}
	}

	pub fn into_literal(self) -> Result<Literal, Id> {
		match self {
			Self::Node(id) => Err(id),
			Self::Literal(l) => Ok(l),
		}
	}

	pub fn into_boolean(self) -> Result<Boolean, Self> {
		self.into_literal()
			.map_err(Self::Node)?
			.into_boolean()
			.map_err(Self::Literal)
	}

	pub fn into_numeric(self) -> Result<Numeric, Self> {
		self.into_literal()
			.map_err(Self::Node)?
			.into_numeric()
			.map_err(Self::Literal)
	}

	pub fn into_integer(self) -> Result<Integer, Self> {
		self.into_numeric()?
			.into_integer()
			.map_err(|v| Self::Literal(Literal::Numeric(v)))
	}

	pub fn into_non_negative_integer(self) -> Result<NonNegativeInteger, Self> {
		self.into_numeric()?
			.into_non_negative_integer()
			.map_err(|v| Self::Literal(Literal::Numeric(v)))
	}

	pub fn into_float(self) -> Result<Float, Self> {
		self.into_numeric()?
			.into_float()
			.map_err(|v| Self::Literal(Literal::Numeric(v)))
	}

	pub fn into_double(self) -> Result<Double, Self> {
		self.into_numeric()?
			.into_double()
			.map_err(|v| Self::Literal(Literal::Numeric(v)))
	}

	pub fn into_lang_string(self) -> Result<LangString, Self> {
		self.into_literal()
			.map_err(Self::Node)?
			.into_lang_string()
			.map_err(Self::Literal)
	}

	pub fn into_string(self) -> Result<String, Self> {
		self.into_literal()
			.map_err(Self::Node)?
			.into_string()
			.map_err(Self::Literal)
	}

	pub fn into_regexp(self) -> Result<RegExp, Self> {
		self.into_literal()
			.map_err(Self::Node)?
			.into_regexp()
			.map_err(Self::Literal)
	}
}

impl<M> TryFrom<vocab::Object<M>> for Value {
	type Error = InvalidLiteral;

	fn try_from(value: vocab::Object<M>) -> Result<Self, Self::Error> {
		match value {
			vocab::Object::Literal(l) => Ok(Value::Literal(l.try_into()?)),
			vocab::Object::Id(id) => Ok(Value::Node(id)),
		}
	}
}

/// Literal value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Literal {
	Boolean(Boolean),
	Numeric(Numeric),
	LangString(LangString),
	String(String),
	Base64Binary(Base64BinaryBuf),
	HexBinary(HexBinaryBuf),
	RegExp(RegExp),
	Other(String, IriIndex),
}

impl Literal {
	pub fn lexical_form(&self) -> Cow<str> {
		match self {
			Self::Boolean(true) => Cow::Borrowed("true"),
			Self::Boolean(false) => Cow::Borrowed("false"),
			Self::Numeric(n) => Cow::Owned(n.to_string()),
			Self::LangString(s) => Cow::Borrowed(s.as_str()),
			Self::String(s) => Cow::Borrowed(s.as_str()),
			Self::Base64Binary(b) => Cow::Owned(b.to_string()),
			Self::HexBinary(b) => Cow::Owned(b.to_string()),
			Self::RegExp(e) => Cow::Owned(e.to_string()),
			Self::Other(s, _) => Cow::Borrowed(s.as_str()),
		}
	}

	pub fn into_boolean(self) -> Result<Boolean, Self> {
		match self {
			Self::Boolean(b) => Ok(b),
			v => Err(v),
		}
	}

	pub fn into_numeric(self) -> Result<Numeric, Self> {
		match self {
			Self::Numeric(n) => Ok(n),
			v => Err(v),
		}
	}

	pub fn into_integer(self) -> Result<Integer, Self> {
		self.into_numeric()?.into_integer().map_err(Self::Numeric)
	}

	pub fn into_non_negative_integer(self) -> Result<NonNegativeInteger, Self> {
		self.into_numeric()?
			.into_non_negative_integer()
			.map_err(Self::Numeric)
	}

	pub fn into_non_positive_integer(self) -> Result<NonPositiveInteger, Self> {
		self.into_numeric()?
			.into_non_positive_integer()
			.map_err(Self::Numeric)
	}

	pub fn into_positive_integer(self) -> Result<PositiveInteger, Self> {
		self.into_numeric()?
			.into_positive_integer()
			.map_err(Self::Numeric)
	}

	pub fn into_negative_integer(self) -> Result<NegativeInteger, Self> {
		self.into_numeric()?
			.into_negative_integer()
			.map_err(Self::Numeric)
	}

	pub fn into_unsigned_long(self) -> Result<UnsignedLong, Self> {
		self.into_numeric()?
			.into_unsigned_long()
			.map_err(Self::Numeric)
	}

	pub fn into_unsigned_int(self) -> Result<UnsignedInt, Self> {
		self.into_numeric()?
			.into_unsigned_int()
			.map_err(Self::Numeric)
	}

	pub fn into_unsigned_short(self) -> Result<UnsignedShort, Self> {
		self.into_numeric()?
			.into_unsigned_short()
			.map_err(Self::Numeric)
	}

	pub fn into_unsigned_byte(self) -> Result<UnsignedByte, Self> {
		self.into_numeric()?
			.into_unsigned_byte()
			.map_err(Self::Numeric)
	}

	pub fn into_long(self) -> Result<Long, Self> {
		self.into_numeric()?.into_long().map_err(Self::Numeric)
	}

	pub fn into_int(self) -> Result<Int, Self> {
		self.into_numeric()?.into_int().map_err(Self::Numeric)
	}

	pub fn into_short(self) -> Result<Short, Self> {
		self.into_numeric()?.into_short().map_err(Self::Numeric)
	}

	pub fn into_byte(self) -> Result<Byte, Self> {
		self.into_numeric()?.into_byte().map_err(Self::Numeric)
	}

	pub fn into_float(self) -> Result<Float, Self> {
		self.into_numeric()?.into_float().map_err(Self::Numeric)
	}

	pub fn into_double(self) -> Result<Double, Self> {
		self.into_numeric()?.into_double().map_err(Self::Numeric)
	}

	pub fn into_lang_string(self) -> Result<LangString, Self> {
		match self {
			Self::LangString(s) => Ok(s),
			v => Err(v),
		}
	}

	pub fn into_string(self) -> Result<String, Self> {
		match self {
			Self::String(s) => Ok(s),
			v => Err(v),
		}
	}

	pub fn into_base64_binary(self) -> Result<Base64BinaryBuf, Self> {
		match self {
			Self::Base64Binary(b) => Ok(b),
			v => Err(v),
		}
	}

	pub fn into_hex_binary(self) -> Result<HexBinaryBuf, Self> {
		match self {
			Self::HexBinary(b) => Ok(b),
			v => Err(v),
		}
	}

	pub fn into_regexp(self) -> Result<RegExp, Self> {
		match self {
			Self::RegExp(e) => Ok(e),
			v => Err(v),
		}
	}
}

macro_rules! from_rational {
	( $($ty:ty),* ) => {
		$(
			impl From<$ty> for Literal {
				fn from(value: $ty) -> Self {
					Self::Numeric(Numeric::Real(Real::Rational(value.into())))
				}
			}
		)*
	};
}

from_rational! {
	Decimal,
	Integer,
	NonNegativeInteger,
	PositiveInteger,
	NonPositiveInteger,
	NegativeInteger,
	Long,
	Int,
	Short,
	Byte,
	UnsignedLong,
	UnsignedInt,
	UnsignedShort,
	UnsignedByte
}

impl From<String> for Literal {
	fn from(value: String) -> Self {
		Self::String(value)
	}
}

impl fmt::Display for Literal {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Boolean(true) => write!(f, "true"),
			Self::Boolean(false) => write!(f, "false"),
			Self::Numeric(n) => n.fmt(f),
			Self::LangString(s) => s.fmt(f),
			Self::String(s) => s.fmt(f),
			Self::Base64Binary(b) => b.fmt(f),
			Self::HexBinary(b) => b.fmt(f),
			Self::RegExp(e) => e.fmt(f),
			Self::Other(s, _) => s.fmt(f),
		}
	}
}

impl<V: IriVocabulary<Iri = IriIndex>> rdf_types::RdfDisplayWithContext<V> for Literal {
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		use rdf_types::RdfDisplay;
		match self {
			Self::Boolean(true) => write!(f, "\"true\"^^{}", vocab::Xsd::Boolean.as_iri()),
			Self::Boolean(false) => write!(f, "\"false\"^^{}", vocab::Xsd::Boolean.as_iri()),
			Self::Numeric(n) => n.rdf_fmt_with(vocabulary, f),
			Self::LangString(s) => s.rdf_fmt(f),
			Self::String(s) => s.fmt(f),
			Self::Base64Binary(b) => write!(f, "\"{b}\"^^{}", vocab::Xsd::Base64Binary.as_iri()),
			Self::HexBinary(b) => write!(f, "\"{b}\"^^{}", vocab::Xsd::HexBinary.as_iri()),
			Self::RegExp(e) => write!(f, "{e}^^{}", vocab::TreeLdr::RegularExpression.as_iri()),
			Self::Other(s, ty) => write!(f, "{s}^^{}", vocabulary.iri(ty).unwrap()),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidLiteral {
	#[error("missing language tag")]
	MissingLanguageTag,

	#[error("invalid lexical value")]
	InvalidLexicalValue(IriIndex),
}

impl<M> TryFrom<vocab::Literal<M>> for Literal {
	type Error = InvalidLiteral;

	fn try_from(value: vocab::Literal<M>) -> Result<Self, Self::Error> {
		macro_rules! match_type {
			( ($s:ident, $ty:ident): $($term:ident),* ) => {
				match $ty {
					IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Boolean)) => {
						match xsd_types::lexical::Boolean::new($s.as_str()) {
							Ok(b) => Ok(Literal::Boolean(b.value())),
							Err(_) => Err(InvalidLiteral::InvalidLexicalValue($ty))
						}
					}
					IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::String)) => {
						Ok(Literal::String($s.into_value()))
					}
					IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Base64Binary)) => {
						match Base64BinaryBuf::decode($s.as_str()) {
							Ok(b) => Ok(Literal::Base64Binary(b)),
							Err(_) => Err(InvalidLiteral::InvalidLexicalValue($ty))
						}
					}
					IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::HexBinary)) => {
						match HexBinaryBuf::decode($s.as_str()) {
							Ok(b) => Ok(Literal::HexBinary(b)),
							Err(_) => Err(InvalidLiteral::InvalidLexicalValue($ty))
						}
					}
					IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::LangString)) => {
						Err(InvalidLiteral::MissingLanguageTag)
					}
					IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::RegularExpression)) => {
						match RegExp::parse($s.as_str()) {
							Ok(e) => Ok(Literal::RegExp(e)),
							Err(_) => Err(InvalidLiteral::InvalidLexicalValue($ty))
						}
					}
					$(
						IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::$term)) => {
							Ok(Integer::parse_rdf(&$s).map_err(|_| InvalidLiteral::InvalidLexicalValue($ty))?.into())
						}
					)*
					ty => Ok(Literal::Other($s.into_value(), ty)),
				}
			};
		}

		match value {
			vocab::Literal::String(s) => Ok(Literal::String(s.into_value())),
			vocab::Literal::TypedString(s, Meta(ty, _)) => match_type! { (s, ty):
				Decimal,
				Integer,
				NonNegativeInteger,
				PositiveInteger,
				NonPositiveInteger,
				NegativeInteger,
				Long,
				Int,
				Short,
				Byte,
				UnsignedLong,
				UnsignedInt,
				UnsignedShort,
				UnsignedByte
			},
			vocab::Literal::LangString(s, tag) => Ok(Literal::LangString(LangString::new(
				s.into_value(),
				tag.into_value(),
			))),
		}
	}
}

pub trait AsRdfLiteral: fmt::Display {
	fn rdf_type(&self) -> IriIndex;

	fn language(&self) -> Option<LanguageTag> {
		None
	}

	fn as_rdf_literal(&self) -> rdf_types::Literal<String, IriIndex> {
		match self.rdf_type() {
			IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::String)) => {
				rdf_types::Literal::String(self.to_string())
			}
			IriIndex::Iri(crate::vocab::Term::Rdf(crate::vocab::Rdf::LangString)) => {
				rdf_types::Literal::LangString(self.to_string(), self.language().unwrap().cloned())
			}
			ty => rdf_types::Literal::TypedString(self.to_string(), ty),
		}
	}
}

impl AsRdfLiteral for Boolean {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Boolean))
	}
}

impl AsRdfLiteral for Real {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Owl(crate::vocab::Owl::Real))
	}
}

impl AsRdfLiteral for Rational {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Owl(crate::vocab::Owl::Real))
	}
}

impl AsRdfLiteral for Float {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Float))
	}
}

impl AsRdfLiteral for Double {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Double))
	}
}

impl AsRdfLiteral for Decimal {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Decimal))
	}
}

impl AsRdfLiteral for Integer {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Integer))
	}
}

impl AsRdfLiteral for NonNegativeInteger {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(
			crate::vocab::Xsd::NonNegativeInteger,
		))
	}
}

impl AsRdfLiteral for PositiveInteger {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::PositiveInteger))
	}
}

impl AsRdfLiteral for NonPositiveInteger {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(
			crate::vocab::Xsd::NonPositiveInteger,
		))
	}
}

impl AsRdfLiteral for NegativeInteger {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::NegativeInteger))
	}
}

impl AsRdfLiteral for Numeric {
	fn rdf_type(&self) -> IriIndex {
		match self {
			Self::Real(r) => r.rdf_type(),
			Self::Double(d) => d.rdf_type(),
			Self::Float(f) => f.rdf_type(),
		}
	}
}

impl AsRdfLiteral for LangString {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Rdf(crate::vocab::Rdf::LangString))
	}

	fn language(&self) -> Option<LanguageTag> {
		Some(self.language())
	}
}

impl AsRdfLiteral for String {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::String))
	}
}

impl AsRdfLiteral for str {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::String))
	}
}

impl AsRdfLiteral for Base64BinaryBuf {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Base64Binary))
	}
}

impl AsRdfLiteral for Base64Binary {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Base64Binary))
	}
}

impl AsRdfLiteral for HexBinaryBuf {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::HexBinary))
	}
}

impl AsRdfLiteral for HexBinary {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::HexBinary))
	}
}

impl AsRdfLiteral for Literal {
	fn language(&self) -> Option<LanguageTag> {
		match self {
			Self::LangString(s) => Some(s.language()),
			_ => None,
		}
	}

	fn rdf_type(&self) -> IriIndex {
		match self {
			Self::Boolean(b) => b.rdf_type(),
			Self::Numeric(n) => n.rdf_type(),
			Self::LangString(s) => s.rdf_type(),
			Self::String(s) => s.rdf_type(),
			Self::Base64Binary(b) => b.rdf_type(),
			Self::HexBinary(b) => b.rdf_type(),
			Self::RegExp(_) => {
				IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::RegularExpression))
			}
			Self::Other(_, ty) => *ty,
		}
	}
}

/// Literal value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralRef<'a> {
	Boolean(Boolean),
	Numeric(Numeric),
	LangString(&'a LangString),
	String(&'a str),
	Base64Binary(&'a Base64Binary),
	HexBinary(&'a HexBinary),
	RegExp(&'a RegExp),
	Other(&'a str, IriIndex),
}

impl<'a> From<&'a Literal> for LiteralRef<'a> {
	fn from(value: &'a Literal) -> Self {
		match value {
			Literal::Boolean(b) => Self::Boolean(*b),
			Literal::Numeric(n) => Self::Numeric(n.clone()),
			Literal::LangString(s) => Self::LangString(s),
			Literal::String(s) => Self::String(s.as_str()),
			Literal::Base64Binary(s) => Self::Base64Binary(s.as_base64_binary()),
			Literal::HexBinary(s) => Self::HexBinary(s.as_hex_binary()),
			Literal::RegExp(e) => Self::RegExp(e),
			Literal::Other(s, ty) => Self::Other(s.as_str(), *ty),
		}
	}
}

impl<'a> From<&'a Boolean> for LiteralRef<'a> {
	fn from(value: &'a Boolean) -> Self {
		Self::Boolean(*value)
	}
}

impl<'a> From<&'a Real> for LiteralRef<'a> {
	fn from(value: &'a Real) -> Self {
		Self::Numeric(value.clone().into())
	}
}

impl<'a> From<&'a Float> for LiteralRef<'a> {
	fn from(value: &'a Float) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a Double> for LiteralRef<'a> {
	fn from(value: &'a Double) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a Decimal> for LiteralRef<'a> {
	fn from(value: &'a Decimal) -> Self {
		Self::Numeric(value.clone().into())
	}
}

impl<'a> From<&'a Integer> for LiteralRef<'a> {
	fn from(value: &'a Integer) -> Self {
		Self::Numeric(value.clone().into())
	}
}

impl<'a> From<&'a NonNegativeInteger> for LiteralRef<'a> {
	fn from(value: &'a NonNegativeInteger) -> Self {
		Self::Numeric(value.clone().into())
	}
}

impl<'a> From<&'a NonPositiveInteger> for LiteralRef<'a> {
	fn from(value: &'a NonPositiveInteger) -> Self {
		Self::Numeric(value.clone().into())
	}
}

impl<'a> From<&'a PositiveInteger> for LiteralRef<'a> {
	fn from(value: &'a PositiveInteger) -> Self {
		Self::Numeric(value.clone().into())
	}
}

impl<'a> From<&'a NegativeInteger> for LiteralRef<'a> {
	fn from(value: &'a NegativeInteger) -> Self {
		Self::Numeric(value.clone().into())
	}
}

impl<'a> From<&'a UnsignedLong> for LiteralRef<'a> {
	fn from(value: &'a UnsignedLong) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a UnsignedInt> for LiteralRef<'a> {
	fn from(value: &'a UnsignedInt) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a UnsignedShort> for LiteralRef<'a> {
	fn from(value: &'a UnsignedShort) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a UnsignedByte> for LiteralRef<'a> {
	fn from(value: &'a UnsignedByte) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a Long> for LiteralRef<'a> {
	fn from(value: &'a Long) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a Int> for LiteralRef<'a> {
	fn from(value: &'a Int) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a Short> for LiteralRef<'a> {
	fn from(value: &'a Short) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a Byte> for LiteralRef<'a> {
	fn from(value: &'a Byte) -> Self {
		Self::Numeric((*value).into())
	}
}

impl<'a> From<&'a String> for LiteralRef<'a> {
	fn from(value: &'a String) -> Self {
		Self::String(value.as_str())
	}
}

impl<'a> From<&'a str> for LiteralRef<'a> {
	fn from(value: &'a str) -> Self {
		Self::String(value)
	}
}

impl<'a> From<&'a Base64BinaryBuf> for LiteralRef<'a> {
	fn from(value: &'a Base64BinaryBuf) -> Self {
		Self::Base64Binary(value.as_base64_binary())
	}
}

impl<'a> From<&'a Base64Binary> for LiteralRef<'a> {
	fn from(value: &'a Base64Binary) -> Self {
		Self::Base64Binary(value)
	}
}

impl<'a> From<&'a HexBinaryBuf> for LiteralRef<'a> {
	fn from(value: &'a HexBinaryBuf) -> Self {
		Self::HexBinary(value.as_hex_binary())
	}
}

impl<'a> From<&'a HexBinary> for LiteralRef<'a> {
	fn from(value: &'a HexBinary) -> Self {
		Self::HexBinary(value)
	}
}

impl<'a> From<&'a Date> for LiteralRef<'a> {
	fn from(_value: &'a Date) -> Self {
		todo!("xsd:date literal")
	}
}

impl<'a> From<&'a Time> for LiteralRef<'a> {
	fn from(_value: &'a Time) -> Self {
		todo!("xsd:time literal")
	}
}

impl<'a> From<&'a DateTime> for LiteralRef<'a> {
	fn from(_value: &'a DateTime) -> Self {
		todo!("xsd:dateTime literal")
	}
}

impl<'a> From<&'a IriBuf> for LiteralRef<'a> {
	fn from(_value: &'a IriBuf) -> Self {
		todo!("tldr:Iri literal")
	}
}

impl<'a> From<&'a UriBuf> for LiteralRef<'a> {
	fn from(_value: &'a UriBuf) -> Self {
		todo!("tldr:Uri literal")
	}
}

impl<'a> From<&'a UrlBuf> for LiteralRef<'a> {
	fn from(_value: &'a UrlBuf) -> Self {
		todo!("tldr:Url literal")
	}
}

impl<'a> From<&'a BytesBuf> for LiteralRef<'a> {
	fn from(_value: &'a BytesBuf) -> Self {
		todo!("tldr:Bytes literal")
	}
}

impl<'a> From<&'a CidBuf> for LiteralRef<'a> {
	fn from(_value: &'a CidBuf) -> Self {
		todo!("tldr:Cid literal")
	}
}

impl<'a> fmt::Display for LiteralRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Boolean(true) => write!(f, "true"),
			Self::Boolean(false) => write!(f, "false"),
			Self::Numeric(n) => n.fmt(f),
			Self::LangString(s) => s.fmt(f),
			Self::String(s) => s.fmt(f),
			Self::Base64Binary(b) => b.fmt(f),
			Self::HexBinary(b) => b.fmt(f),
			Self::RegExp(e) => e.fmt(f),
			Self::Other(s, _) => s.fmt(f),
		}
	}
}

impl<'a, V: IriVocabulary<Iri = IriIndex>> rdf_types::RdfDisplayWithContext<V> for LiteralRef<'a> {
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		use rdf_types::RdfDisplay;
		match self {
			Self::Boolean(true) => write!(f, "\"true\"^^{}", vocab::Xsd::Boolean.as_iri()),
			Self::Boolean(false) => write!(f, "\"false\"^^{}", vocab::Xsd::Boolean.as_iri()),
			Self::Numeric(n) => n.rdf_fmt_with(vocabulary, f),
			Self::LangString(s) => s.rdf_fmt(f),
			Self::String(s) => s.fmt(f),
			Self::Base64Binary(b) => write!(f, "\"{b}\"^^{}", vocab::Xsd::Base64Binary.as_iri()),
			Self::HexBinary(b) => write!(f, "\"{b}\"^^{}", vocab::Xsd::HexBinary.as_iri()),
			Self::RegExp(e) => write!(f, "{e}^^{}", vocab::TreeLdr::RegularExpression.as_iri()),
			Self::Other(s, ty) => write!(f, "{s}^^{}", vocabulary.iri(ty).unwrap()),
		}
	}
}

impl<'a> AsRdfLiteral for LiteralRef<'a> {
	fn language(&self) -> Option<LanguageTag> {
		match self {
			Self::LangString(s) => Some(s.language()),
			_ => None,
		}
	}

	fn rdf_type(&self) -> IriIndex {
		match self {
			Self::Boolean(b) => b.rdf_type(),
			Self::Numeric(n) => n.rdf_type(),
			Self::LangString(s) => s.rdf_type(),
			Self::String(s) => s.rdf_type(),
			Self::Base64Binary(b) => b.rdf_type(),
			Self::HexBinary(b) => b.rdf_type(),
			Self::RegExp(_) => {
				IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::RegularExpression))
			}
			Self::Other(_, ty) => *ty,
		}
	}
}
