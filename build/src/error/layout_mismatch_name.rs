use treeldr::{Id, Vocabulary, vocab::Display, Name};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutMismatchName<M> {
	pub id: Id,
	pub expected: Name,
	pub found: Name,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMismatchName<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("name mismatch for layout `{}`", self.id.display(vocab))
	}
}