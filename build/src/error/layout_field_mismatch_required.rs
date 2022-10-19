use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutFieldMismatchRequired<M> {
	pub id: Id,
	pub expected: bool,
	pub found: bool,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMismatchRequired<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("required status mismatch for layout field `{}`", self.id.display(vocab))
	}
}