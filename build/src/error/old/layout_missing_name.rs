use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutMissingName(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMissingName {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("no name defined for layout `{}`", self.0.with(vocab))
	}

	fn notes(&self, _vocab: &TldrVocabulary) -> Vec<String> {
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