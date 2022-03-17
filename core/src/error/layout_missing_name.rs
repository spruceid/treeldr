use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutMissingName(pub Id);

impl<F> super::AnyError<F> for LayoutMissingName {
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