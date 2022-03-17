use crate::{Id, Vocabulary, vocab::{Object, Display}};
use locspan::Location;

#[derive(Debug)]
pub struct ListMismatchItem<F> {
	pub id: Id,
	pub expected: Object<F>,
	pub found: Object<F>,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for ListMismatchItem<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("item mismatch for list `{}`", self.id.display(vocab))
	}
}