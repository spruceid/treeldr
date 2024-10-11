use core::fmt;
use std::io::{self, BufRead, Write};

use clap::builder::TypedValueParser;
use json_syntax::Print;
use treeldr_layouts::{value::NonJsonValue, Registry};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
	#[error("JSON parse error: {0}")]
	Json(json_syntax::parse::Error<io::Error>),

	#[error("CBOR parse error: {0}")]
	Cbor(serde_cbor::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
	#[error(transparent)]
	NonJsonValue(NonJsonValue),

	#[error(transparent)]
	IO(#[from] io::Error),

	#[error("invalid CBOR tag: {0}")]
	CborTag(treeldr_layouts::value::cbor::InvalidTag),

	#[error(transparent)]
	Cbor(serde_cbor::Error),
}

#[derive(Debug, Clone)]
pub enum TreeFormat {
	Json,
	Cbor,
}

impl TreeFormat {
	pub const POSSIBLE_VALUES: &'static [&'static str] =
		&["application/json", "json", "application/cbor", "cbor"];

	pub fn parser(
	) -> clap::builder::MapValueParser<clap::builder::PossibleValuesParser, fn(String) -> Self> {
		clap::builder::PossibleValuesParser::new(Self::POSSIBLE_VALUES)
			.map(|s| Self::new(&s).unwrap())
	}

	pub fn new(name: &str) -> Option<Self> {
		match name {
			"application/json" | "json" => Some(Self::Json),
			"application/cbor" | "cbor" => Some(Self::Cbor),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Json => "application/json",
			Self::Cbor => "application/cbor",
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
			Self::Cbor => serde_cbor::from_reader(input).map_err(LoadError::Cbor),
		}
	}

	pub fn write_typed(
		&self,
		layouts: &impl Registry,
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
			Self::Cbor => {
				let cbor = value
					.try_into_tagged_serde_cbor(layouts)
					.map_err(WriteError::CborTag)?;
				serde_cbor::to_writer(output, &cbor).map_err(WriteError::Cbor)
			}
		}
	}

	pub fn write_untyped(
		&self,
		value: treeldr_layouts::Value,
		pretty: bool,
		mut output: impl Write,
	) -> Result<(), WriteError> {
		match self {
			Self::Json => {
				let json: json_syntax::Value =
					value.try_into().map_err(WriteError::NonJsonValue)?;
				if pretty {
					write!(output, "{}", json.pretty_print()).map_err(WriteError::IO)
				} else {
					write!(output, "{}", json.compact_print()).map_err(WriteError::IO)
				}
			}
			Self::Cbor => serde_cbor::to_writer(output, &value).map_err(WriteError::Cbor),
		}
	}
}

impl fmt::Display for TreeFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
