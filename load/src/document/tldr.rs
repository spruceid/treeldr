use locspan::{Location, Meta};
use rdf_types::{Generator, VocabularyMut};
use treeldr::{BlankIdIndex, IriIndex};
use treeldr_syntax as syntax;

use crate::{source, BuildContext, DisplayPath, LangError, ParseError};

/// TreeLDR document.
pub struct Document {
	doc: syntax::Document<source::Metadata>,
	local_context: syntax::build::LocalContext<source::Metadata>,
}

impl Document {
	pub fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), syntax::build::Error<source::Metadata>> {
		use treeldr_build::Document;
		self.doc
			.declare(&mut self.local_context, context, vocabulary, generator)
	}

	pub fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		mut self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), syntax::build::Error<source::Metadata>> {
		use treeldr_build::Document;
		self.doc
			.define(&mut self.local_context, context, vocabulary, generator)
	}
}

impl From<treeldr_syntax::build::Error<source::Metadata>> for LangError {
	fn from(e: treeldr_syntax::build::Error<source::Metadata>) -> Self {
		Self::TreeLdr(e)
	}
}

/// Import a TreeLDR file.
pub fn import<'f, P>(
	files: &'f source::Files<P>,
	source_id: source::FileId,
) -> Result<Document, Meta<ParseError, source::Metadata>>
where
	P: DisplayPath<'f>,
{
	use syntax::Parse;
	let file = files.get(source_id).unwrap();

	log::debug!("ready for parsing.");
	match syntax::Document::parse_str(file.buffer().as_str(), |span| {
		source::Metadata::Extern(Location::new(source_id, span))
	}) {
		Ok(doc) => {
			log::debug!("parsing succeeded.");
			Ok(Document {
				doc: doc.into_value(),
				local_context: syntax::build::LocalContext::new(
					file.base_iri().map(|iri| iri.into()),
				),
			})
		}
		Err(e) => Err(e.map(ParseError::TreeLdr)),
	}
}
