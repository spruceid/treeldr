use crate::Options;
use iref::{Iri, IriBuf};
use rdf_types::Vocabulary;
use std::fmt;
use treeldr::{layout, BlankIdIndex, IriIndex, Ref};

#[derive(clap::Args)]
/// Generate a JSON-LD Context from a TreeLDR model.
pub struct Command {
	/// Layout schemas to generate.
	layouts: Vec<IriBuf>,

	/// Use layout name as `rdf:type` value.
	#[clap(long = "rdf-type-to-layout-name")]
	rdf_type_to_layout_name: bool,
}

pub enum Error<F> {
	UndefinedLayout(IriBuf),
	NotALayout(IriBuf, treeldr::node::TypesMetadata<F>),
	Generation(crate::Error),
}

impl<F> fmt::Display for Error<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UndefinedLayout(iri) => write!(f, "undefined layout `{}`", iri),
			Self::NotALayout(iri, _) => write!(f, "node `{}` is not a layout", iri),
			Self::Generation(e) => e.fmt(f),
		}
	}
}

fn find_layout<M: Clone>(
	vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	model: &treeldr::Model<M>,
	iri: Iri,
) -> Result<Ref<layout::Definition<M>>, Box<Error<M>>> {
	let name = vocabulary
		.get(iri)
		.ok_or_else(|| Error::UndefinedLayout(iri.into()))?;
	model
		.require_layout(treeldr::Id::Iri(name))
		.map_err(|e| match e {
			treeldr::Error::NodeUnknown(_) => Box::new(Error::UndefinedLayout(iri.into())),
			treeldr::Error::NodeInvalidType(e) => Box::new(Error::NotALayout(iri.into(), e.found)),
		})
}

impl Command {
	pub fn execute<F: Clone>(
		self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		model: &treeldr::Model<F>,
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
		model: &treeldr::Model<F>,
	) -> Result<(), Box<Error<F>>> {
		let mut layouts = Vec::with_capacity(self.layouts.len());
		for layout_iri in self.layouts {
			layouts.push(find_layout(vocabulary, model, layout_iri.as_iri())?);
		}

		let options = Options {
			rdf_type_to_layout_name: self.rdf_type_to_layout_name,
		};

		match crate::generate(vocabulary, model, options, layouts) {
			Ok(definition) => {
				use json_ld::syntax::Print;
				println!("{}", definition.pretty_print());

				Ok(())
			}
			Err(e) => Err(Box::new(Error::Generation(e))),
		}
	}
}
