use core::fmt;
use std::fmt::Write;

use iref::IriBuf;
use rdf_types::{BlankIdBuf, LocalTerm, RdfDisplay, Term};

use crate::{Registry, TypeRef};

pub struct DisplayContext<'a, G: Registry> {
	pub registry: &'a G,
	pub self_: Option<TypeRef<G::PackageId>>,
	pub indent: Indent,
}

impl<G: Registry> DisplayContext<'_, G> {
	pub fn inner(&self) -> Self {
		Self {
			registry: self.registry,
			self_: self.self_,
			indent: Indent(self.indent.0 + 1),
		}
	}

	pub fn inner_self(&self, self_: TypeRef<G::PackageId>) -> Self {
		Self {
			registry: self.registry,
			self_: Some(self_),
			indent: Indent(self.indent.0 + 1),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Indent(pub usize);

impl fmt::Display for Indent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for _ in 0..self.0 {
			f.write_char('\t')?;
		}

		Ok(())
	}
}

pub trait DisplaySignature<P, R> {
	fn display_signature<'a, G>(&'a self, registry: &'a G) -> DisplayedSignature<'a, G, Self>
	where
		G: Registry<PackageId = P, Resource = R>,
	{
		self.display_signature_with(DisplayContext {
			registry,
			self_: None,
			indent: Indent(0),
		})
	}

	fn display_signature_with<'a, G>(
		&'a self,
		context: DisplayContext<'a, G>,
	) -> DisplayedSignature<'a, G, Self>
	where
		G: Registry<PackageId = P, Resource = R>,
	{
		DisplayedSignature { context, t: self }
	}

	fn fmt_signature(
		&self,
		context: &DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result;
}

pub struct DisplayedSignature<'a, G: Registry, T: ?Sized> {
	context: DisplayContext<'a, G>,
	t: &'a T,
}

impl<G, T> fmt::Display for DisplayedSignature<'_, G, T>
where
	G: Registry,
	T: ?Sized + DisplaySignature<G::PackageId, G::Resource>,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.t.fmt_signature(&self.context, f)
	}
}

pub trait DisplayResource {
	fn fmt_resource(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl DisplayResource for Term {
	fn fmt_resource(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Iri(id) => id.fmt_resource(f),
			Self::Literal(l) => l.fmt_resource(f),
		}
	}
}

impl DisplayResource for LocalTerm {
	fn fmt_resource(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Anonymous(blank_id) => blank_id.rdf_fmt(f),
			Self::Named(named) => named.fmt_resource(f),
		}
	}
}

impl DisplayResource for IriBuf {
	fn fmt_resource(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "<{}>", self)
	}
}

impl DisplayResource for BlankIdBuf {
	fn fmt_resource(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self)
	}
}

impl DisplayResource for rdf_types::Literal {
	fn fmt_resource(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		string_literal(&self.value, StringLiteralDelimiter::Single, f)?;
		if self.type_.is_xsd_string() {
			Ok(())
		} else {
			self.type_.rdf_fmt(f)
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringLiteralDelimiter {
	Single,
	Double,
}

impl StringLiteralDelimiter {
	fn char(self) -> char {
		match self {
			Self::Single => '\'',
			Self::Double => '"',
		}
	}
}

/// Formats a string literal according to [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-strings).
pub fn string_literal(
	s: &str,
	delimiter: StringLiteralDelimiter,
	f: &mut fmt::Formatter,
) -> fmt::Result {
	use fmt::Display;
	f.write_char(delimiter.char())?;

	for c in s.chars() {
		match c {
			'\\' => f.write_str("\\\\")?,
			'\'' if matches!(delimiter, StringLiteralDelimiter::Single) => f.write_str("\\\'")?,
			'\"' if matches!(delimiter, StringLiteralDelimiter::Double) => f.write_str("\\\"")?,
			'\u{0008}' => f.write_str("\\b")?,
			'\u{0009}' => f.write_str("\\t")?,
			'\u{000a}' => f.write_str("\\n")?,
			'\u{000c}' => f.write_str("\\f")?,
			'\u{000d}' => f.write_str("\\r")?,
			'\u{0000}'..='\u{001f}' => {
				f.write_str("\\u")?;

				let codepoint = c as u32;
				let d = codepoint & 0x000f;
				let c = (codepoint & 0x00f0) >> 4;
				let b = (codepoint & 0x0f00) >> 8;
				let a = (codepoint & 0xf000) >> 12;

				digit(a).fmt(f)?;
				digit(b).fmt(f)?;
				digit(c).fmt(f)?;
				digit(d).fmt(f)?
			}
			_ => c.fmt(f)?,
		}
	}

	f.write_char(delimiter.char())
}

fn digit(c: u32) -> char {
	match c {
		0x0 => '0',
		0x1 => '1',
		0x2 => '2',
		0x3 => '3',
		0x4 => '4',
		0x5 => '5',
		0x6 => '6',
		0x7 => '7',
		0x8 => '8',
		0x9 => '9',
		0xa => 'a',
		0xb => 'b',
		0xc => 'c',
		0xd => 'd',
		0xe => 'e',
		0xf => 'f',
		_ => panic!("invalid input: {}", c),
	}
}
