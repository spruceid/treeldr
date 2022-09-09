use treeldr::{Id, Vocabulary, Name, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutFieldMismatchName<M> {
	pub id: Id,
	pub expected: Name,
	pub found: Name,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMismatchName<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("name mismatch for layout field `{}`", self.id.display(vocab))
	}
}