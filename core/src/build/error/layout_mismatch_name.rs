use crate::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutMismatchName<F> {
	pub id: Id,
	pub expected: String,
	pub found: String,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutMismatchName<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("name mismatch for layout `{}`", self.id.display(vocab))
	}
}