use iref::{IriBuf, IriRef, IriRefBuf};
use std::fmt;
use treeldr::{layout, Ref};

#[derive(clap::Args)]
/// Generate a JSON-LD Context from a TreeLDR model.
pub struct Command {
	#[clap(multiple_occurrences(true))]
	/// Layout schema to generate.
	layout: IriRefBuf,
}

pub enum Error {
	UndefinedLayout(IriBuf),
	NotALayout(IriBuf, treeldr::node::CausedTypes),
	Serialization(serde_json::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UndefinedLayout(iri) => write!(f, "undefined layout `{}`", iri),
			Self::NotALayout(iri, _) => write!(f, "node `{}` is not a layout", iri),
			Self::Serialization(e) => write!(f, "JSON serialization failed: {}", e),
		}
	}
}

fn find_layout(model: &treeldr::Model, iri_ref: IriRef) -> Result<Ref<layout::Definition>, Error> {
	let iri = iri_ref.resolved(model.base_iri());
	let id = model
		.vocabulary()
		.id(&iri)
		.ok_or_else(|| Error::UndefinedLayout(iri.clone()))?;
	model.require_layout(id, None).map_err(|e| match e.inner() {
		treeldr::Error::UnknownNode { .. } => Error::UndefinedLayout(iri.clone()),
		treeldr::Error::InvalidNodeType { found, .. } => Error::NotALayout(iri.clone(), *found),
		_ => unreachable!(),
	})
}

impl Command {
	pub fn execute(self, model: &treeldr::Model) {
		log::info!("generating JSON Schema.");
		match self.try_execute(model) {
			Ok(()) => (),
			Err(e) => {
				log::error!("{}", e);
				std::process::exit(1)
			}
		}
	}

	fn try_execute(self, model: &treeldr::Model) -> Result<(), Error> {
		let layout_ref = find_layout(model, self.layout.as_iri_ref())?;
		match crate::generate(model, layout_ref) {
			Ok(()) => Ok(()),
			Err(crate::Error::Serialization(e)) => Err(Error::Serialization(e)),
		}
	}
}