use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutFieldMissingLayout(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMissingLayout {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no layout defined for field `{}`", self.0.display(vocab))
	}
}