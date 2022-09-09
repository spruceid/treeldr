use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutMissingName(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMissingName {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no name defined for layout `{}`", self.0.display(vocab))
	}

	fn notes(&self, _vocab: &Vocabulary) -> Vec<String> {
		match self.0 {
			Id::Blank(_) => {
				vec!["layout name cannot be derived from a blank node identifier.".to_string()]
			}
			Id::Iri(_) => {
				vec!["layout name cannot be derived from its IRI.".to_string()]
			}
		}
	}
}