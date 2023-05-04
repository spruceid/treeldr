use locspan::{Meta, Location};

use crate::{source, ParseError, DisplayPath};

/// Import a N-Quads file.
pub fn import<'f, P>(
	files: &'f source::Files<P>,
	source_id: source::FileId,
) -> Result<nquads_syntax::Document<source::Metadata>, Meta<ParseError, source::Metadata>>
where
	P: DisplayPath<'f>,
{
	use nquads_syntax::Parse;
	let file = files.get(source_id).unwrap();
	match nquads_syntax::Document::parse_str(file.buffer().as_str(), |span| {
		source::Metadata::Extern(Location::new(source_id, span))
	}) {
		Ok(Meta(doc, _)) => {
			log::debug!("parsing succeeded.");
			Ok(doc)
		}
		Err(Meta(e, meta)) => Err(Meta(ParseError::NQuads(e), meta)),
	}
}