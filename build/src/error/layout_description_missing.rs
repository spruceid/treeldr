use treeldr::{Id, vocab::TldrVocabulary};
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutDescriptionMissing(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutDescriptionMissing {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("no implementation for layout `{}`", self.0.with(vocab))
	}
}