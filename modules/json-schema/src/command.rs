use crate::embedding;
use embedding::Embedding;
use iref::{Iri, IriBuf};
use std::fmt;
use treeldr::{layout, vocab::Display, Ref, Vocabulary};

#[derive(clap::Args)]
/// Generate a JSON Schema from a TreeLDR model.
pub struct Command {
	#[clap(multiple_occurrences(true), required(true))]
	/// Layout schema to generate.
	layouts: Vec<IriBuf>,

	#[clap(short = 'e', multiple_occurrences(true))]
	/// Layout schema to embed.
	embeds: Vec<IriBuf>,

	#[clap(short = 't', long = "type")]
	/// Add a property in each schema, with the given name,
	/// storing the type the object.
	type_property: Option<String>,
}

pub struct NotALayoutError<F>(pub IriBuf, pub treeldr::node::CausedTypes<F>);

pub enum Error<F> {
	NoLayoutName(String),
	UndefinedLayout(IriBuf),
	NotALayout(Box<NotALayoutError<F>>),
	InfiniteSchema(String),
	Serialization(serde_json::Error),
}

impl<F> fmt::Display for Error<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::NoLayoutName(iri) => write!(f, "layout `{}` has no name", iri),
			Self::UndefinedLayout(iri) => write!(f, "undefined layout `{}`", iri),
			Self::NotALayout(e) => write!(f, "node `{}` is not a layout", e.0),
			Self::InfiniteSchema(iri) => write!(f, "infinite schema `{}`", iri),
			Self::Serialization(e) => write!(f, "JSON serialization failed: {}", e),
		}
	}
}

fn find_layout<F: Clone>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	iri: Iri,
) -> Result<Ref<layout::Definition<F>>, Box<Error<F>>> {
	let name = treeldr::vocab::Term::try_from_iri(iri, vocabulary)
		.ok_or_else(|| Error::UndefinedLayout(iri.into()))?;
	model
		.require_layout(treeldr::Id::Iri(name))
		.map_err(|e| match e {
			treeldr::Error::NodeUnknown(_) => Box::new(Error::UndefinedLayout(iri.into())),
			treeldr::Error::NodeInvalidType(e) => Box::new(Error::NotALayout(Box::new(
				NotALayoutError(iri.into(), e.found),
			))),
		})
}

impl Command {
	pub fn execute<F: Clone>(self, vocabulary: &Vocabulary, model: &treeldr::Model<F>) {
		log::info!("generating JSON Schema.");
		match self.try_execute(vocabulary, model) {
			Ok(()) => (),
			Err(e) => {
				log::error!("{}", e);
				std::process::exit(1)
			}
		}
	}

	fn try_execute<F: Clone>(
		self,
		vocabulary: &Vocabulary,
		model: &treeldr::Model<F>,
	) -> Result<(), Box<Error<F>>> {
		// Find the layouts to generate.
		let mut layouts = Vec::new();

		for iri in self.layouts {
			layouts.push(find_layout(vocabulary, model, iri.as_iri())?);
		}

		layouts.reverse();

		let main_layout_ref = layouts.pop().unwrap();

		// Build the embedding configuration.
		let mut embedding_config = embedding::Configuration::new();
		for &layout_ref in &layouts {
			embedding_config.set(layout_ref, Embedding::Indirect);
		}
		for iri in &self.embeds {
			let layout_ref = find_layout(vocabulary, model, iri.as_iri())?;
			embedding_config.set(layout_ref, Embedding::Direct);
		}

		match crate::generate(
			vocabulary,
			model,
			&embedding_config,
			self.type_property.as_deref(),
			main_layout_ref,
		) {
			Ok(()) => Ok(()),
			Err(crate::Error::NoLayoutName(r)) => Err(Box::new(Error::NoLayoutName(
				model
					.layouts()
					.get(r)
					.unwrap()
					.id()
					.display(vocabulary)
					.to_string(),
			))),
			Err(crate::Error::InfiniteSchema(r)) => Err(Box::new(Error::InfiniteSchema(
				model
					.layouts()
					.get(r)
					.unwrap()
					.id()
					.display(vocabulary)
					.to_string(),
			))),
			Err(crate::Error::Serialization(e)) => Err(Box::new(Error::Serialization(e))),
		}
	}
}
