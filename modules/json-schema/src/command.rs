use crate::embedding;
use contextual::WithContext;
use embedding::Embedding;
use iref::{Iri, IriBuf};
use json_syntax::Print;
use rdf_types::Vocabulary;
use std::fmt;
use treeldr::{BlankIdIndex, IriIndex, TId};

#[derive(clap::Args)]
/// Generate a JSON Schema from a TreeLDR model.
pub struct Command {
	#[clap(required(true))]
	/// Layout schema to generate.
	layouts: Vec<IriBuf>,

	#[clap(short = 'e')]
	/// Layout schema to embed.
	embeds: Vec<IriBuf>,

	#[clap(short = 't', long = "type")]
	/// Add a property in each schema, with the given name,
	/// storing the type the object.
	type_property: Option<String>,
}

pub struct NotALayoutError<M>(
	pub IriBuf,
	pub treeldr::PropertyValues<TId<treeldr::Type>, M>,
);

pub enum Error<M> {
	NoLayoutName(String),
	UndefinedLayout(IriBuf),
	NotALayout(Box<NotALayoutError<M>>),
	InfiniteSchema(String),
}

impl<F> fmt::Display for Error<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::NoLayoutName(iri) => write!(f, "layout `{iri}` has no name"),
			Self::UndefinedLayout(iri) => write!(f, "undefined layout `{iri}`"),
			Self::NotALayout(e) => write!(f, "node `{}` is not a layout", e.0),
			Self::InfiniteSchema(iri) => write!(f, "infinite schema `{iri}`"),
		}
	}
}

fn find_layout<F: Clone>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::MutableModel<F>,
	iri: Iri,
) -> Result<TId<treeldr::Layout>, Box<Error<F>>> {
	let name = vocabulary
		.get(iri)
		.ok_or_else(|| Error::UndefinedLayout(iri.into()))?;
	let id: TId<treeldr::Layout> = treeldr::TId::new(treeldr::Id::Iri(name));
	model.require(id).map_err(|e| match e {
		treeldr::Error::NodeUnknown(_) => Box::new(Error::UndefinedLayout(iri.into())),
		treeldr::Error::NodeInvalidType(e) => Box::new(Error::NotALayout(Box::new(
			NotALayoutError(iri.into(), e.found),
		))),
	})?;
	Ok(id)
}

impl Command {
	pub fn execute<F: Clone>(
		self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		model: &treeldr::MutableModel<F>,
	) {
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
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		model: &treeldr::MutableModel<F>,
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
			Ok(json_schema) => {
				println!("{}", json_schema.pretty_print());

				Ok(())
			}
			Err(crate::Error::NoLayoutName(r)) => Err(Box::new(Error::NoLayoutName(
				model.get(r).unwrap().id().with(vocabulary).to_string(),
			))),
			Err(crate::Error::InfiniteSchema(r)) => Err(Box::new(Error::InfiniteSchema(
				model.get(r).unwrap().id().with(vocabulary).to_string(),
			))),
		}
	}
}
