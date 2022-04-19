use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutMismatchType<F> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutMismatchType<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("type mismatch for layout `{}`", self.id.display(vocab))
	}
}