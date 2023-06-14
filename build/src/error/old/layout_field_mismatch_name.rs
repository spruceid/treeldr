use treeldr::{Id, Name, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutFieldMismatchName<M> {
	pub id: Id,
	pub expected: Name,
	pub found: Name,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMismatchName<M> {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("name mismatch for layout field `{}`", self.id.with(vocab))
	}
}