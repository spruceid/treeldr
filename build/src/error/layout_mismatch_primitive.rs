use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutMismatchPrimitive<M> {
	pub id: Id,
	pub expected: treeldr::layout::Primitive,
	pub found: treeldr::layout::Primitive,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMismatchPrimitive<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("datatype primitive mismatch for layout `{}`", self.id.display(vocab))
	}
}