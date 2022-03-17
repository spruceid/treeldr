use iref::{Iri, IriBuf};
use std::fmt;
use treeldr::{layout, Ref};

#[derive(clap::Args)]
/// Generate a JSON-LD Context from a TreeLDR model.
pub struct Command {
	#[clap(multiple_occurrences(true))]
	/// Layout schema to generate.
	layout: IriBuf,
}

pub enum Error<F> {
	UndefinedLayout(IriBuf),
	NotALayout(IriBuf, treeldr::node::CausedTypes<F>),
	Serialization(serde_json::Error),
}

impl<F> fmt::Display for Error<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UndefinedLayout(iri) => write!(f, "undefined layout `{}`", iri),
			Self::NotALayout(iri, _) => write!(f, "node `{}` is not a layout", iri),
			Self::Serialization(e) => write!(f, "JSON serialization failed: {}", e),
		}
	}
}

fn find_layout<F: Clone>(
	model: &treeldr::Model<F>,
	iri: Iri,
) -> Result<Ref<layout::Definition<F>>, Error<F>> {
	let name = treeldr::vocab::Name::try_from_iri(iri, model.vocabulary())
		.ok_or_else(|| Error::UndefinedLayout(iri.into()))?;
	model
		.require_layout(treeldr::Id::Iri(name))
		.map_err(|e| match e {
			treeldr::error::Description::NodeUnknown(_) => Error::UndefinedLayout(iri.into()),
			treeldr::error::Description::NodeInvalidType(e) => {
				Error::NotALayout(iri.into(), e.found)
			}
			_ => unreachable!(),
		})
}

impl Command {
	pub fn execute<F: Clone>(self, model: &treeldr::Model<F>) {
		log::info!("generating JSON Schema.");
		match self.try_execute(model) {
			Ok(()) => (),
			Err(e) => {
				log::error!("{}", e);
				std::process::exit(1)
			}
		}
	}

	fn try_execute<F: Clone>(self, model: &treeldr::Model<F>) -> Result<(), Error<F>> {
		let layout_ref = find_layout(model, self.layout.as_iri())?;
		match crate::generate(model, layout_ref) {
			Ok(()) => Ok(()),
			Err(crate::Error::Serialization(e)) => Err(Error::Serialization(e)),
		}
	}
}
