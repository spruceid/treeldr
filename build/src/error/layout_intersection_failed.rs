use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutIntersectionFailed {
	pub id: Id
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutIntersectionFailed {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("intersection `{}` failed", self.id.display(vocab))
	}
}