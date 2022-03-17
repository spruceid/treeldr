use crate::{Id, Vocabulary, vocab::Display, build::layout::Description};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutMismatchDescription<F> {
	pub id: Id,
	pub expected: Description,
	pub found: Description,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutMismatchDescription<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("implementation mismatch for layout `{}`", self.id.display(vocab))
	}
}