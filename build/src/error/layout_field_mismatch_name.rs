use treeldr::{Id, Vocabulary, Name, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutFieldMismatchName<F> {
	pub id: Id,
	pub expected: Name,
	pub found: Name,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutFieldMismatchName<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("name mismatch for layout field `{}`", self.id.display(vocab))
	}
}