use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct ListMissingItem(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for ListMissingItem {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("missing item for list `{}`", self.0.with(vocab))
	}
}