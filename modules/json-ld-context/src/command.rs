use crate::Options;
use iref::{Iri, IriBuf};
use json_ld::{Context, ContextLoader, Process};
use locspan::Span;
use rdf_types::{Vocabulary, VocabularyMut};
use std::{
	fmt, io,
	path::{Path, PathBuf},
	str::FromStr,
};
use treeldr::{BlankIdIndex, IriIndex, TId};

mod loader;
pub use loader::FsLoader;

pub trait Files: Send + Sync {
	type Id: Clone;
	type Metadata: Clone + Send + Sync;

	fn load(&mut self, path: &Path, base_iri: IriBuf) -> Result<(Self::Id, &str), io::Error>;

	fn build_metadata(&self, id: Self::Id, span: Span) -> Self::Metadata;
}

#[derive(clap::Args)]
/// Generate a JSON-LD Context from a TreeLDR model.
pub struct Command {
	/// Layout schemas to generate.
	layouts: Vec<IriBuf>,

	/// File system mount points.
	#[clap(short = 'm', long = "mount")]
	mount_points: Vec<MountPoint>,

	/// Extern contexts to import.
	#[clap(short = 'c', long = "context")]
	contexts: Vec<IriBuf>,

	/// Use layout name as `rdf:type` value.
	#[clap(long = "rdf-type-to-layout-name")]
	rdf_type_to_layout_name: bool,
}

#[derive(Debug)]
pub enum InvalidMountPointSyntax {
	MissingSeparator(String),
	InvalidIri(String, iref::Error),
}

impl std::error::Error for InvalidMountPointSyntax {}

impl fmt::Display for InvalidMountPointSyntax {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::MissingSeparator(s) => write!(f, "missing separator `=` in `{s}`"),
			Self::InvalidIri(i, e) => write!(f, "invalid IRI `{i}`: {e}"),
		}
	}
}

pub struct MountPoint {
	pub iri: IriBuf,
	pub path: PathBuf,
}

impl FromStr for MountPoint {
	type Err = InvalidMountPointSyntax;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split_once('=') {
			Some((iri, path)) => {
				let iri = IriBuf::new(iri)
					.map_err(|e| InvalidMountPointSyntax::InvalidIri(iri.to_string(), e))?;
				Ok(Self {
					path: path.into(),
					iri,
				})
			}
			None => Err(InvalidMountPointSyntax::MissingSeparator(s.to_string())),
		}
	}
}

pub enum Error<E, M> {
	UndefinedLayout(IriBuf),
	NotALayout(IriBuf, treeldr::Multiple<TId<treeldr::Type>, M>),
	Generation(crate::GenerateError<E, M>),
	ExternContextLoadFailed(IriBuf),
}

impl<E, M> fmt::Display for Error<E, M> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UndefinedLayout(iri) => write!(f, "undefined layout `{}`", iri),
			Self::NotALayout(iri, _) => write!(f, "node `{}` is not a layout", iri),
			Self::Generation(e) => e.fmt(f),
			Self::ExternContextLoadFailed(iri) => {
				write!(f, "unable to load extern context `{iri}`")
			}
		}
	}
}

fn find_layout<E, M: Clone>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::Model<M>,
	iri: Iri,
) -> Result<TId<treeldr::Layout>, Box<Error<E, M>>> {
	let name = vocabulary
		.get(iri)
		.ok_or_else(|| Error::UndefinedLayout(iri.into()))?;
	let id: TId<treeldr::Layout> = TId::new(treeldr::Id::Iri(name));
	model.require(id).map_err(|e| match e {
		treeldr::Error::NodeUnknown(_) => Box::new(Error::UndefinedLayout(iri.into())),
		treeldr::Error::NodeInvalidType(e) => Box::new(Error::NotALayout(iri.into(), e.found)),
	})?;
	Ok(id)
}

impl Command {
	pub async fn execute<V, M>(
		self,
		vocabulary: &mut V,
		files: &mut impl Files<Metadata = M>,
		model: &treeldr::Model<M>,
	) where
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex> + Send + Sync,
		M: Clone + Send + Sync,
	{
		log::info!("generating JSON-LD context...");
		match self.try_execute(vocabulary, files, model).await {
			Ok(()) => (),
			Err(e) => {
				log::error!("{}", e);
				std::process::exit(1)
			}
		}
	}

	async fn try_execute<V, M>(
		self,
		vocabulary: &mut V,
		files: &mut impl Files<Metadata = M>,
		model: &treeldr::Model<M>,
	) -> Result<(), Box<Error<loader::ContextError<M>, M>>>
	where
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex> + Send + Sync,
		M: Clone + Send + Sync,
	{
		let mut layouts = Vec::with_capacity(self.layouts.len());
		for layout_iri in self.layouts {
			layouts.push(find_layout(vocabulary, model, layout_iri.as_iri())?);
		}

		let mut loader = FsLoader::new(files);

		for m in self.mount_points {
			loader.mount(vocabulary.insert(m.iri.as_iri()), m.path);
		}

		let mut options = Options {
			rdf_type_to_layout_name: self.rdf_type_to_layout_name,
			context: Context::new(None),
		};

		for iri in self.contexts {
			let i = vocabulary.insert(iri.as_iri());
			match loader.load_context_with(vocabulary, i).await {
				Ok(local_context) => {
					let local_context = local_context.into_document();
					let processed = local_context
						.process_with(
							vocabulary,
							&options.context,
							&mut loader,
							None,
							json_ld::context_processing::Options::default(),
						)
						.await;

					match processed {
						Ok(processed) => options.context = processed.into_processed(),
						Err(_) => return Err(Box::new(Error::ExternContextLoadFailed(iri))),
					}
				}
				Err(_) => return Err(Box::new(Error::ExternContextLoadFailed(iri))),
			}
		}

		match crate::generate(vocabulary, &mut loader, model, options, &layouts).await {
			Ok(definition) => {
				use json_ld::syntax::Print;
				println!("{}", definition.pretty_print());

				Ok(())
			}
			Err(e) => Err(Box::new(Error::Generation(e))),
		}
	}
}
