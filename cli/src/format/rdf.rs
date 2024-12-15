use core::fmt;
use std::{
	io::{self, BufRead, Write},
	str::FromStr,
};

use nquads_syntax::Parse;
use treeldr::{
	rdf::{dataset::BTreeDataset, Quad},
	syntax::Span,
};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
	#[error("N-Quads parse error: {0}")]
	NQuads(
		#[from] nquads_syntax::parsing::MetaError<nquads_syntax::lexing::Error<io::Error>, Span>,
	),
}

#[derive(Debug, Default, Clone)]
pub enum RdfFormat {
	#[default]
	NQuads,
}

impl RdfFormat {
	pub fn new(name: &str) -> Option<Self> {
		match name {
			"nq" | "nquads" | "n-quads" | "application/n-quads" => Some(Self::NQuads),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::NQuads => "application/n-quads",
		}
	}

	pub fn load(&self, input: impl BufRead) -> Result<BTreeDataset, LoadError> {
		match self {
			Self::NQuads => {
				let utf8_input = utf8_decode::UnsafeDecoder::new(input.bytes());
				let document = nquads_syntax::GrdfDocument::parse_utf8(utf8_input)
					.map_err(LoadError::NQuads)?
					.into_value();
				Ok(document
					.into_iter()
					.map(|q| nquads_syntax::strip_quad(q.into_value()))
					.collect())
			}
		}
	}

	pub fn write(
		&self,
		dataset: impl IntoIterator<Item = Quad>,
		mut output: impl Write,
	) -> Result<(), io::Error> {
		match self {
			Self::NQuads => {
				for quad in dataset {
					writeln!(output, "{} .", quad)?;
				}

				Ok(())
			}
		}
	}
}

impl FromStr for RdfFormat {
	type Err = InvalidRdfFormat;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(s).ok_or(InvalidRdfFormat)
	}
}

impl fmt::Display for RdfFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

#[derive(Debug, thiserror::Error)]
#[error("invalid RDF dataset format")]
pub struct InvalidRdfFormat;
