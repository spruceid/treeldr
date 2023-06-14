use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutFieldMismatchFunctional<M> {
	pub id: Id,
	pub expected: bool,
	pub found: bool,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMismatchFunctional<M> {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("functional status mismatch for layout field `{}`", self.id.with(vocab))
	}
}