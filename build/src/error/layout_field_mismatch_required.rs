use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutFieldMismatchRequired<F> {
	pub id: Id,
	pub expected: bool,
	pub found: bool,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutFieldMismatchRequired<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("required status mismatch for layout field `{}`", self.id.display(vocab))
	}
}