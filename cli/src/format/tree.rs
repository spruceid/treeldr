use core::fmt;
use std::{
	io::{self, BufRead, Write},
	str::FromStr,
};

use clap::builder::TypedValueParser;
use json_syntax::Print;
use treeldr::value::{serde::json::NonJsonValue, TypedValue, ValueParseError};

use crate::{PackageId, Registry};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
	#[error(transparent)]
	IO(#[from] io::Error),

	#[error("TRON parse error: {0}")]
	Tron(#[from] ValueParseError),

	#[error("JSON parse error: {0}")]
	Json(json_syntax::parse::Error<io::Error>),
	// #[error("CBOR parse error: {0}")]
	// Cbor(serde_cbor::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
	#[error(transparent)]
	NonJsonValue(NonJsonValue),

	#[error(transparent)]
	IO(#[from] io::Error),
	// #[error("invalid CBOR tag: {0}")]
	// CborTag(treeldr_layouts::value::cbor::InvalidTag),

	// #[error(transparent)]
	// Cbor(serde_cbor::Error),
}

#[derive(Debug, Clone)]
pub enum TreeFormat {
	Tron,
	Json,
	// Cbor,
}

impl TreeFormat {
	pub const POSSIBLE_VALUES: &'static [&'static str] = &[
		"application/tron",
		"tron",
		"application/json",
		"json",
		"application/cbor",
		"cbor",
	];

	pub fn parser(
	) -> clap::builder::MapValueParser<clap::builder::PossibleValuesParser, fn(String) -> Self> {
		clap::builder::PossibleValuesParser::new(Self::POSSIBLE_VALUES)
			.map(|s| Self::new(&s).unwrap())
	}

	pub fn new(name: &str) -> Option<Self> {
		match name {
			"application/tron" | "tron" => Some(Self::Tron),
			"application/json" | "json" => Some(Self::Json),
			// "application/cbor" | "cbor" => Some(Self::Cbor),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Tron => "application/tron",
			Self::Json => "application/json",
			// Self::Cbor => "application/cbor",
		}
	}

	pub fn load(&self, mut input: impl BufRead) -> Result<treeldr::Value, LoadError> {
		match self {
			Self::Tron => {
				let mut buf = String::new();
				input.read_to_string(&mut buf)?;
				treeldr::Value::from_str(&buf).map_err(Into::into)
			}
			Self::Json => {
				use json_syntax::Parse;
				let utf8_input = utf8_decode::UnsafeDecoder::new(input.bytes());
				let (json, _) =
					json_syntax::Value::parse_utf8(utf8_input).map_err(LoadError::Json)?;
				Ok(json.into())
			} // Self::Cbor => serde_cbor::from_reader(input).map_err(LoadError::Cbor),
		}
	}

	pub fn write(
		&self,
		_registry: &Registry,
		value: &TypedValue<PackageId>,
		pretty: bool,
		mut output: impl Write,
	) -> Result<(), WriteError> {
		match self {
			Self::Tron => {
				writeln!(output, "{}", value.to_untyped().to_string()).map_err(WriteError::IO)
			}
			Self::Json => {
				let json: json_syntax::Value = value
					.clone()
					.into_untyped()
					.try_into()
					.map_err(WriteError::NonJsonValue)?;
				if pretty {
					writeln!(output, "{}", json.pretty_print()).map_err(WriteError::IO)
				} else {
					writeln!(output, "{}", json.compact_print()).map_err(WriteError::IO)
				}
			} // Self::Cbor => {
			  // 	let cbor = value
			  // 		.try_into_tagged_serde_cbor(registry)
			  // 		.map_err(WriteError::CborTag)?;
			  // 	serde_cbor::to_writer(output, &cbor).map_err(WriteError::Cbor)
			  // }
		}
	}

	// pub fn write_untyped(
	// 	&self,
	// 	value: &treeldr::Value,
	// 	pretty: bool,
	// 	mut output: impl Write,
	// ) -> Result<(), WriteError> {
	// 	match self {
	// 		Self::Json => {
	// 			let json: json_syntax::Value =
	// 				value.clone().try_into().map_err(WriteError::NonJsonValue)?;
	// 			if pretty {
	// 				write!(output, "{}", json.pretty_print()).map_err(WriteError::IO)
	// 			} else {
	// 				write!(output, "{}", json.compact_print()).map_err(WriteError::IO)
	// 			}
	// 		}
	// 		// Self::Cbor => serde_cbor::to_writer(output, &value).map_err(WriteError::Cbor),
	// 	}
	// }
}

impl fmt::Display for TreeFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
