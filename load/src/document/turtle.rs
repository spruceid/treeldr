use locspan::{Meta, Location};

use crate::{source, ParseError, DisplayPath, LangError};

impl From<turtle_syntax::build::MetaError<source::Metadata>> for LangError {
	fn from(e: turtle_syntax::build::MetaError<source::Metadata>) -> Self {
		Self::Turtle(e)
	}
}

/// Import a RDF Turtle file.
pub fn import<'f, P>(
	files: &'f source::Files<P>,
	source_id: source::FileId,
) -> Result<turtle_syntax::Document<source::Metadata>, Meta<ParseError, source::Metadata>>
where
	P: DisplayPath<'f>,
{
	use turtle_syntax::Parse;
	let file = files.get(source_id).unwrap();
	match turtle_syntax::Document::parse_str(file.buffer().as_str(), |span| {
		source::Metadata::Extern(Location::new(source_id, span))
	}) {
		Ok(Meta(doc, _)) => {
			log::debug!("parsing succeeded.");
			Ok(doc)
		}
		Err(Meta(e, meta)) => Err(Meta(ParseError::Turtle(e), meta)),
	}
}