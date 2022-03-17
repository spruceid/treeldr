use crate::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutFieldMismatchLayout<F> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutFieldMismatchLayout<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("layout mismatch for layout field `{}`", self.id.display(vocab))
	}
}