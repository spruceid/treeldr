use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutFieldMissingName(pub Id);

impl<F> super::AnyError<F> for LayoutFieldMissingName {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no name defined for field `{}`", self.0.display(vocab))
	}

	fn notes(&self, _vocab: &Vocabulary) -> Vec<String> {
		match self.0 {
			Id::Blank(_) => {
				vec!["field name could not be derived from a blank node identifier.".to_string()]
			}
			Id::Iri(_) => {
				vec!["field name could not be derived from its IRI.".to_string()]
			}
		}
	}
}