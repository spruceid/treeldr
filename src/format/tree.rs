use core::fmt;
use std::io::{self, BufRead, Write};

use clap::builder::TypedValueParser;
use json_syntax::Print;
use treeldr_layouts::value::NonJsonValue;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
	#[error("JSON parse error: {0}")]
	Json(json_syntax::parse::Error<io::Error>),
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
	#[error(transparent)]
	NonJsonValue(NonJsonValue),

	#[error(transparent)]
	IO(#[from] io::Error),
}

#[derive(Debug, Clone)]
pub enum TreeFormat {
	Json,
}

impl TreeFormat {
	pub const POSSIBLE_VALUES: &'static [&'static str] = &["application/json", "json"];

	pub fn parser(
	) -> clap::builder::MapValueParser<clap::builder::PossibleValuesParser, fn(String) -> Self> {
		clap::builder::PossibleValuesParser::new(Self::POSSIBLE_VALUES)
			.map(|s| Self::new(&s).unwrap())
	}

	pub fn new(name: &str) -> Option<Self> {
		match name {
			"json" | "application/json" => Some(Self::Json),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Json => "application/json",
		}
	}

	pub fn load(&self, input: impl BufRead) -> Result<treeldr_layouts::Value, LoadError> {
		match self {
			Self::Json => {
				use json_syntax::Parse;
				let utf8_input = utf8_decode::UnsafeDecoder::new(input.bytes());
				let (json, _) =
					json_syntax::Value::parse_utf8(utf8_input).map_err(LoadError::Json)?;
				Ok(json.into())
			}
		}
	}

	pub fn write(
		&self,
		value: treeldr_layouts::TypedValue,
		pretty: bool,
		mut output: impl Write,
	) -> Result<(), WriteError> {
		match self {
			Self::Json => {
				let json: json_syntax::Value = value
					.into_untyped()
					.try_into()
					.map_err(WriteError::NonJsonValue)?;
				if pretty {
					write!(output, "{}", json.pretty_print()).map_err(WriteError::IO)
				} else {
					write!(output, "{}", json.compact_print()).map_err(WriteError::IO)
				}
			}
		}
	}
}

impl fmt::Display for TreeFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
