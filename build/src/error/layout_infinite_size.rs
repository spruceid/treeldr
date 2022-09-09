use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutInfiniteSize {
	pub id: Id
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutInfiniteSize {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("the size of layout `{}` is infinite", self.id.display(vocab))
	}
}