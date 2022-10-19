use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutFieldMismatchLayout<M> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMismatchLayout<M> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("layout mismatch for layout field `{}`", self.id.with(vocab))
	}
}