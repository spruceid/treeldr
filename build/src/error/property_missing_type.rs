use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct PropertyMissingType(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for PropertyMissingType {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no range defined for property `{}`", self.0.display(vocab))
	}
}