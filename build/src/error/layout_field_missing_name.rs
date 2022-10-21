use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutFieldMissingName(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMissingName {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("no name defined for field `{}`", self.0.with(vocab))
	}

	fn notes(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> Vec<String> {
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