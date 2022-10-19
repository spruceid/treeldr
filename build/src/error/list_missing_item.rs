use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct ListMissingItem(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for ListMissingItem {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("missing item for list `{}`", self.0.display(vocab))
	}
}