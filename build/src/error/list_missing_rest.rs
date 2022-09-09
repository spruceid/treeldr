use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct ListMissingRest(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for ListMissingRest {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("missing rest for list `{}`", self.0.display(vocab))
	}
}