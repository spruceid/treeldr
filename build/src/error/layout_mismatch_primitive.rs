use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct LayoutMismatchPrimitive<F> {
	pub id: Id,
	pub expected: treeldr::layout::Primitive,
	pub found: treeldr::layout::Primitive,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for LayoutMismatchPrimitive<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("datatype primitive mismatch for layout `{}`", self.id.display(vocab))
	}
}