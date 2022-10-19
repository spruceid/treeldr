use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutVariantMissingName(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutVariantMissingName {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no name defined for variant `{}`", self.0.display(vocab))
	}

	fn notes(&self, _vocab: &Vocabulary) -> Vec<String> {
		match self.0 {
			Id::Blank(_) => {
				vec!["variant name could not be derived from a blank node identifier.".to_string()]
			}
			Id::Iri(_) => {
				vec!["variant name could not be derived from its IRI.".to_string()]
			}
		}
	}
}