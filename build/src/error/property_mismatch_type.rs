use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct PropertyMismatchType<M> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for PropertyMismatchType<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("type mismatch for property `{}`", self.id.display(vocab))
	}
}