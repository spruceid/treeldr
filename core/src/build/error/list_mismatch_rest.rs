use crate::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct ListMismatchRest<F> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for ListMismatchRest<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("rest mismatch for list `{}`", self.id.display(vocab))
	}
}