use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct PropertyMismatchType<M> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for PropertyMismatchType<M> {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("type mismatch for property `{}`", self.id.with(vocab))
	}
}