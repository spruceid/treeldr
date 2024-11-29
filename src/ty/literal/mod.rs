mod boolean;
use core::fmt;
use std::cmp::Ordering;

pub use boolean::*;

pub mod number;
pub use number::Number;

mod byte_string;
pub use byte_string::ByteString;

mod text_string;
pub use text_string::TextString;

use crate::{display::DisplaySignature, Literal};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralType {
	Unit,
	Boolean(Boolean),
	Number(Number),
	ByteString(ByteString),
	TextString(TextString),
}

impl LiteralType {
	pub fn boolean() -> Self {
		Self::Boolean(Boolean::default())
	}

	pub fn number() -> Self {
		Self::Number(Number::default())
	}

	pub fn int() -> Self {
		Self::Number(Number::int())
	}

	pub fn uint() -> Self {
		Self::Number(Number::uint())
	}

	pub fn byte_string() -> Self {
		Self::ByteString(ByteString)
	}

	pub fn text_string() -> Self {
		Self::TextString(TextString::default())
	}

	pub fn contains(&self, value: &Literal) -> bool {
		match (self, value) {
			(Self::Unit, Literal::Unit) => true,
			(Self::Boolean(ty), Literal::Boolean(b)) => ty.contains(*b),
			(Self::Number(ty), Literal::Number(n)) => ty.contains(n),
			(Self::ByteString(ty), Literal::ByteString(b)) => ty.contains(b),
			(Self::TextString(ty), Literal::TextString(t)) => ty.contains(t),
			_ => false,
		}
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(Self::Unit, Self::Unit) => Some(Ordering::Equal),
			(Self::Boolean(a), Self::Boolean(b)) => a.subtype_cmp(b),
			(Self::Number(a), Self::Number(b)) => a.subtype_cmp(b),
			(Self::ByteString(a), Self::ByteString(b)) => a.subtype_cmp(b),
			(Self::TextString(a), Self::TextString(b)) => a.subtype_cmp(b),
			_ => None,
		}
	}
}

impl<P, R> DisplaySignature<P, R> for LiteralType {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl crate::Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self {
			Self::Unit => write!(f, "null"),
			Self::Boolean(ty) => ty.fmt_signature(context, f),
			Self::Number(ty) => ty.fmt_signature(context, f),
			Self::ByteString(ty) => ty.fmt_signature(context, f),
			Self::TextString(ty) => ty.fmt_signature(context, f),
		}
	}
}
