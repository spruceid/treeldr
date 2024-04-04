use core::fmt;
use std::{
	io::{self, BufRead, Write},
	str::FromStr,
};

use locspan::Span;
use nquads_syntax::Parse;
use rdf_types::dataset::BTreeDataset;

#[derive(Debug, thiserror::Error)]
#[error("unknown RDF format `{0}`")]
pub struct UnknownRDFFormat(String);

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
	#[error("N-Quads parse error: {0}")]
	NQuads(
		#[from] nquads_syntax::parsing::MetaError<nquads_syntax::lexing::Error<io::Error>, Span>,
	),
}

#[derive(Debug, Clone)]
pub enum RDFFormat {
	NQuads,
}

impl RDFFormat {
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

	pub fn write(&self, dataset: BTreeDataset, mut output: impl Write) -> Result<(), io::Error> {
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

impl fmt::Display for RDFFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl FromStr for RDFFormat {
	type Err = UnknownRDFFormat;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"nq" | "nquads" | "n-quads" | "application/n-quads" => Ok(Self::NQuads),
			_ => Err(UnknownRDFFormat(s.to_owned())),
		}
	}
}
