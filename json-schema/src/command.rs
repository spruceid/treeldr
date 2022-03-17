use crate::embedding;
use embedding::Embedding;
use iref::{IriBuf, IriRef, IriRefBuf};
use std::fmt;
use treeldr::{layout, Ref};

#[derive(clap::Args)]
/// Generate a JSON Schema from a TreeLDR model.
pub struct Command {
	#[clap(multiple_occurrences(true))]
	/// Layout schema to generate.
	layouts: Vec<IriRefBuf>,

	#[clap(short = 'e', multiple_occurrences(true))]
	/// Layout schema to embed.
	embeds: Vec<IriRefBuf>,
}

pub enum Error<F> {
	InvalidLayoutIri(IriBuf),
	UndefinedLayout(IriBuf),
	NotALayout(IriBuf, treeldr::node::CausedTypes<F>),
	InfiniteSchema(IriBuf),
	Serialization(serde_json::Error),
}

impl fmt::Display for Error<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidLayoutIri(iri) => write!(f, "invalid layout IRI `{}`", iri),
			Self::UndefinedLayout(iri) => write!(f, "undefined layout `{}`", iri),
			Self::NotALayout(iri, _) => write!(f, "node `{}` is not a layout", iri),
			Self::InfiniteSchema(iri) => write!(f, "infinite schema `{}`", iri),
			Self::Serialization(e) => write!(f, "JSON serialization failed: {}", e),
		}
	}
}

fn find_layout<F>(model: &treeldr::Model<F>, iri_ref: IriRef) -> Result<Ref<layout::Definition<F>>, Error<F>> {
	let iri = iri_ref.resolved(model.base_iri());
	let id = model
		.vocabulary()
		.id(&iri)
		.ok_or_else(|| Error::UndefinedLayout(iri.clone()))?;
	model.require_layout(id, None).map_err(|e| match e.inner() {
		treeldr::error::Description::NodeUnknown(_) => Error::UndefinedLayout(iri.clone()),
		treeldr::error::Description::NodeInvalidType(e) => Error::NotALayout(iri.clone(), *e.found),
		_ => unreachable!(),
	})
}

impl Command {
	pub fn execute<F>(self, model: &treeldr::Model<F>) {
		log::info!("generating JSON Schema.");
		match self.try_execute(model) {
			Ok(()) => (),
			Err(e) => {
				log::error!("{}", e);
				std::process::exit(1)
			}
		}
	}

	fn try_execute<F>(self, model: &treeldr::Model<F>) -> Result<(), Error<F>> {
		// Find the layouts to generate.
		let mut layouts = Vec::new();

		for iri_ref in self.layouts {
			layouts.push(find_layout(model, iri_ref.as_iri_ref())?);
		}

		layouts.reverse();

		let main_layout_ref = layouts.pop().unwrap();

		// Build the embedding configuration.
		let mut embedding_config = embedding::Configuration::new();
		for &layout_ref in &layouts {
			embedding_config.set(layout_ref, Embedding::Indirect);
		}
		for iri_ref in &self.embeds {
			let layout_ref = find_layout(model, iri_ref.as_iri_ref())?;
			embedding_config.set(layout_ref, Embedding::Direct);
		}

		match crate::generate(model, &embedding_config, main_layout_ref) {
			Ok(()) => Ok(()),
			Err(crate::Error::InvalidLayoutIri(iri)) => Err(Error::InvalidLayoutIri(iri)),
			Err(crate::Error::InfiniteSchema(r)) => {
				Err(Error::InfiniteSchema(layout_iri(model, r)))
			}
			Err(crate::Error::Serialization(e)) => Err(Error::Serialization(e)),
		}
	}
}

fn layout_iri<F>(model: &treeldr::Model<F>, r: Ref<layout::Definition<F>>) -> IriBuf {
	let layout = model.layouts().get(r).unwrap();
	model.vocabulary().get(layout.id()).unwrap().into()
}
