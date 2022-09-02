use iref::{Iri, IriBuf};
use std::fmt;
use treeldr::{layout, Ref, Vocabulary};

#[derive(clap::Args)]
/// Generate a JSON-LD Context from a TreeLDR model.
pub struct Command {
	#[clap(multiple_occurrences(true))]
	/// Layout schemas to generate.
	layouts: Vec<IriBuf>
}

pub enum Error<F> {
	UndefinedLayout(IriBuf),
	NotALayout(IriBuf, treeldr::node::CausedTypes<F>),
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

fn find_layout<F: Clone>(
	vocabulary: &Vocabulary,
	model: &treeldr::Model<F>,
	iri: Iri,
) -> Result<Ref<layout::Definition<F>>, Error<F>> {
	let name = treeldr::vocab::Term::try_from_iri(iri, vocabulary)
		.ok_or_else(|| Error::UndefinedLayout(iri.into()))?;
	model
		.require_layout(treeldr::Id::Iri(name))
		.map_err(|e| match e {
			treeldr::Error::NodeUnknown(_) => Error::UndefinedLayout(iri.into()),
			treeldr::Error::NodeInvalidType(e) => Error::NotALayout(iri.into(), e.found),
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
	) -> Result<(), Error<F>> {
		let mut layouts = Vec::with_capacity(self.layouts.len());
		for layout_iri in self.layouts {
			layouts.push(find_layout(vocabulary, model, layout_iri.as_iri())?);
		}

		match crate::generate(vocabulary, model, layouts) {
			Ok(definition) => {
				use json_ld::syntax::Print;
				println!("{}", definition.pretty_print());

				Ok(())
			},
			Err(e) => Err(Error::Generation(e)),
		}
	}
}
