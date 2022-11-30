use futures::future::{BoxFuture, FutureExt};
use json_ld::syntax::Parse;
use json_ld::{Loader, RemoteDocument};
use locspan::Meta;
use rdf_types::IriVocabulary;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use treeldr::IriIndex;

use super::Files;

// pub type ContextError<M> =
// 	json_ld::loader::ContextLoaderError<Error<M>, Meta<json_ld::loader::ExtractContextError<M>, M>>;

#[derive(Debug)]
pub enum Error<M> {
	NoMountPoint,
	IO(std::io::Error),
	Parse(Meta<json_ld::syntax::parse::Error<M>, M>),
}

impl<E: fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::NoMountPoint => write!(f, "no mount point"),
			Self::IO(e) => e.fmt(f),
			Self::Parse(e) => e.fmt(f),
		}
	}
}

/// File-system loader.
///
/// This is a special JSON-LD document loader that can load document from the file system by
/// attaching a directory to specific URLs.
pub struct FsLoader<'f, F: Files> {
	mount_points: HashMap<PathBuf, IriIndex>,
	files: &'f mut F,
	cache: HashMap<IriIndex, RemoteDocument<IriIndex, F::Metadata>>,
}

fn filepath(
	mount_points: &HashMap<PathBuf, IriIndex>,
	vocabulary: &impl IriVocabulary<Iri = IriIndex>,
	url: IriIndex,
) -> Option<PathBuf> {
	let url = vocabulary.iri(&url).unwrap();
	for (path, target_url) in mount_points {
		if let Some((suffix, _, _)) = url.suffix(vocabulary.iri(target_url).unwrap().as_iri_ref()) {
			let mut filepath = path.clone();
			for seg in suffix.as_path().segments() {
				filepath.push(seg.as_str())
			}

			return Some(filepath);
		}
	}

	None
}

impl<'f, F: Files> FsLoader<'f, F> {
	#[inline(always)]
	pub fn mount<P: AsRef<Path>>(&mut self, url: IriIndex, path: P) {
		self.mount_points.insert(path.as_ref().into(), url);
	}
}

impl<'f, F: Files> Loader<IriIndex, F::Metadata> for FsLoader<'f, F> {
	type Output = json_ld::syntax::Value<F::Metadata>;
	type Error = Error<F::Metadata>;

	fn load_with<'a>(
		&'a mut self,
		vocabulary: &'a (impl Sync + IriVocabulary<Iri = IriIndex>),
		url: IriIndex,
	) -> BoxFuture<'a, Result<RemoteDocument<IriIndex, F::Metadata>, Self::Error>>
	where
		IriIndex: 'a,
	{
		async move {
			match self.cache.entry(url) {
				std::collections::hash_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
				std::collections::hash_map::Entry::Vacant(entry) => {
					match filepath(&self.mount_points, vocabulary, url) {
						Some(filepath) => {
							let (id, content) = self
								.files
								.load(&filepath, vocabulary.iri(&url).unwrap().to_owned())
								.map_err(Error::IO)?;
							let content = content.to_owned();
							let json = json_ld::syntax::Value::parse_str(&content, |span| {
								self.files.build_metadata(id.clone(), span)
							})
							.map_err(Error::Parse)?;
							let doc = RemoteDocument::new(Some(url), json);
							entry.insert(doc.clone());
							Ok(doc)
						}
						None => Err(Error::NoMountPoint),
					}
				}
			}
		}
		.boxed()
	}
}

impl<'f, F: Files> FsLoader<'f, F> {
	pub fn new(files: &'f mut F) -> Self {
		Self {
			mount_points: HashMap::new(),
			files,
			cache: HashMap::new(),
		}
	}
}
