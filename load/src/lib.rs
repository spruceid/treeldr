use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use iref::IriBuf;
use locspan::{Location, MaybeLocated, Meta};
use rdf_types::{Generator, Vocabulary, VocabularyMut};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use thiserror::Error;
use treeldr::{
	reporting::Diagnose,
	vocab::{GraphLabel, Object},
	BlankIdIndex, Id, IriIndex,
};

mod error;
mod source;
mod document;

pub use error::*;
pub use source::*;
pub use document::Document;

pub use treeldr::reporting;
pub type BuildContext = treeldr_build::Context<source::Metadata>;

/// Build all the given documents.
pub fn build_all<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	build_context: &mut BuildContext,
	documents: Vec<Document>,
) -> Result<treeldr::Model<source::Metadata>, BuildAllError> {
	build_context.apply_built_in_definitions(vocabulary, generator);

	let mut declared_documents = Vec::with_capacity(documents.len());
	for doc in documents {
		declared_documents.push(
			doc.declare(build_context, vocabulary, generator)
				.map_err(BuildAllError::Declaration)?,
		)
	}

	for doc in declared_documents {
		doc.build(build_context, vocabulary, generator)
			.map_err(BuildAllError::Link)?
	}

	build_context
		.build(vocabulary, generator)
		.map_err(BuildAllError::Build)
}

/// RDF dataset.
pub type Dataset =
	grdf::meta::BTreeDataset<Id, Id, Object<source::Metadata>, GraphLabel, source::Metadata>;
