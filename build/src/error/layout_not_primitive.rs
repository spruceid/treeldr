use treeldr::{Id, vocab::TldrVocabulary};
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutNotPrimitive(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutNotPrimitive {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("layout `{}` is not a primitive layout", self.0.with(vocab))
	}
}