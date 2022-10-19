use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct PropertyMismatchFunctional<M> {
	pub id: Id,
	pub expected: bool,
	pub found: bool,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for PropertyMismatchFunctional<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("functional status mismatch for property `{}`", self.id.display(vocab))
	}
}