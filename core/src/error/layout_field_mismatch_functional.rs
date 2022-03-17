use crate::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutFieldMismatchFunctional<F> {
	pub id: Id,
	pub expected: bool,
	pub found: bool,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutFieldMismatchFunctional<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("functional status mismatch for layout field `{}`", self.id.display(vocab))
	}
}