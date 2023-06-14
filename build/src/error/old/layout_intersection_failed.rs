use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutIntersectionFailed {
	pub id: Id
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutIntersectionFailed {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("intersection `{}` failed", self.id.with(vocab))
	}
}