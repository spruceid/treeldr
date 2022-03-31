use crate::{Id, Vocabulary, vocab::Display, vocab::Name};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutMismatchName<F> {
	pub id: Id,
	pub expected: Name,
	pub found: Name,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutMismatchName<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("name mismatch for layout `{}`", self.id.display(vocab))
	}
}