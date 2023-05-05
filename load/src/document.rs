use std::{hash::Hash, path::Path};

use iref::IriBuf;
use locspan::Meta;
use rdf_types::{Generator, Id, VocabularyMut};
use treeldr::{BlankIdIndex, IriIndex};

use crate::{source, BuildContext, Dataset, DisplayPath, FileId, LangError, LoadError, MimeType};

pub mod json;
pub mod nquads;
pub mod tldr;
#[cfg(feature = "turtle")]
pub mod turtle;

pub enum Document {
	TreeLdr(Box<tldr::Document>),

	NQuads(nquads_syntax::Document<source::Metadata>),

	#[cfg(feature = "turtle")]
	Turtle(turtle_syntax::Document<source::Metadata>),

	Json(Box<json::Document>),
}

pub enum DeclaredDocument {
	TreeLdr(Box<tldr::Document>),

	NQuads(Dataset),

	#[cfg(feature = "turtle")]
	Turtle(Dataset),

	Json(Box<json::Document>),
}

impl Document {
	/// Load the document located at the given `path`.
	pub fn load<'f, P>(
		files: &'f mut source::Files<P>,
		filename: &Path,
	) -> Result<(Self, source::FileId), LoadError>
	where
		P: Clone + Eq + Hash + DisplayPath<'f> + for<'a> From<&'a Path>,
	{
		match files.load(&filename, None, None) {
			Ok(file_id) => {
				let document = match files.get(file_id).unwrap().mime_type() {
					Some(type_) => Self::from_file_id(files, file_id, type_)?,
					None => return Err(LoadError::UnrecognizedFormat(filename.to_owned())),
				};

				Ok((document, file_id))
			}
			Err(e) => Err(LoadError::UnableToRead(filename.to_owned(), e)),
		}
	}

	/// Lead a document from its content.
	pub fn load_content<'f, P>(
		files: &'f mut source::Files<P>,
		source: P,
		content: String,
		base_iri: Option<IriBuf>,
		type_: MimeType,
	) -> Result<(Self, source::FileId), LoadError>
	where
		P: Clone + Eq + Hash + DisplayPath<'f>,
	{
		let file_id = files.load_content(source, base_iri, Some(type_), content);
		let document = Self::from_file_id(files, file_id, type_)?;
		Ok((document, file_id))
	}

	fn from_file_id<'f, P>(
		files: &'f mut source::Files<P>,
		file_id: FileId,
		type_: MimeType,
	) -> Result<Self, LoadError>
	where
		P: DisplayPath<'f>,
	{
		match type_ {
			MimeType::TreeLdr => Ok(Self::TreeLdr(Box::new(tldr::import(files, file_id)?))),
			MimeType::NQuads => Ok(Self::NQuads(nquads::import(files, file_id)?)),
			#[cfg(feature = "turtle")]
			MimeType::Turtle => Ok(Self::Turtle(turtle::import(files, file_id)?)),
			#[cfg(feature = "json-schema")]
			MimeType::Json(t) => Ok(Self::Json(Box::new(json::import(files, file_id, t)?))),
			#[allow(unreachable_patterns)]
			mime_type => Err(LoadError::UnsupportedMimeType(mime_type)),
		}
	}

	pub fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<DeclaredDocument, LangError> {
		match self {
			Self::TreeLdr(mut d) => {
				d.declare(context, vocabulary, generator)?;
				Ok(DeclaredDocument::TreeLdr(d))
			}
			Self::NQuads(d) => {
				let dataset: Dataset = d
					.into_iter()
					.map(|Meta(quad, meta)| {
						Meta(
							quad.insert_into(vocabulary)
								.map_predicate(|Meta(p, m)| Meta(Id::Iri(p), m)),
							meta,
						)
					})
					.collect();

				use treeldr_build::Document;
				dataset
					.declare(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
				Ok(DeclaredDocument::NQuads(dataset))
			}
			#[cfg(feature = "turtle")]
			Self::Turtle(d) => {
				let dataset: Dataset = d
					.build_triples_with(None, vocabulary, &mut *generator)?
					.into_iter()
					.map(|Meta(triple, meta)| {
						Meta(
							triple
								.map_predicate(|Meta(p, m)| Meta(Id::Iri(p), m))
								.into_quad(None),
							meta,
						)
					})
					.collect();

				use treeldr_build::Document;
				dataset
					.declare(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
				Ok(DeclaredDocument::NQuads(dataset))
			}
			Self::Json(d) => {
				d.declare(context, vocabulary, generator)?;
				Ok(DeclaredDocument::Json(d))
			}
		}
	}
}

impl DeclaredDocument {
	pub fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		context: &mut BuildContext,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), LangError> {
		match self {
			Self::TreeLdr(d) => {
				d.build(context, vocabulary, generator)?;
				Ok(())
			}
			Self::NQuads(d) => {
				use treeldr_build::Document;
				d.define(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
				Ok(())
			}
			#[cfg(feature = "turtle")]
			Self::Turtle(d) => {
				use treeldr_build::Document;
				d.define(&mut (), context, vocabulary, generator)
					.map_err(LangError::NQuads)?;
				Ok(())
			}
			Self::Json(_) => Ok(()),
		}
	}
}
