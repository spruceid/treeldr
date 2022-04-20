use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutMismatchDescription<F> {
	pub id: Id,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutMismatchDescription<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("implementation mismatch for layout `{}`", self.id.display(vocab))
	}
}