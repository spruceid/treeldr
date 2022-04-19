use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct PropertyMismatchType<F> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for PropertyMismatchType<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("type mismatch for property `{}`", self.id.display(vocab))
	}
}