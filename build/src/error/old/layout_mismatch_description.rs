use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutMismatchDescription<M> {
	pub id: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMismatchDescription<M> {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("implementation mismatch for layout `{}`", self.id.with(vocab))
	}
}