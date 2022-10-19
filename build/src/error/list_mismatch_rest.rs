use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct ListMismatchRest<M> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for ListMismatchRest<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("rest mismatch for list `{}`", self.id.display(vocab))
	}
}