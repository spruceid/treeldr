use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutFieldMismatchLayout<M> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMismatchLayout<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("layout mismatch for layout field `{}`", self.id.display(vocab))
	}
}