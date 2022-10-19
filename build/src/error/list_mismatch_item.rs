use treeldr::{Id, Vocabulary, vocab::{Object, Display}};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct ListMismatchItem<M> {
	pub id: Id,
	pub expected: Object<M>,
	pub found: Object<M>,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for ListMismatchItem<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("item mismatch for list `{}`", self.id.display(vocab))
	}
}