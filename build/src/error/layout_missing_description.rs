use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutMissingDescription(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMissingDescription {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no implementation for layout `{}`", self.0.display(vocab))
	}
}