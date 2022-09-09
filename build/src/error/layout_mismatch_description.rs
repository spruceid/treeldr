use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutMismatchDescription<M> {
	pub id: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMismatchDescription<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("implementation mismatch for layout `{}`", self.id.display(vocab))
	}
}