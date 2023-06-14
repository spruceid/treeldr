use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutMissingDescription(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMissingDescription {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("no implementation for layout `{}`", self.0.with(vocab))
	}
}