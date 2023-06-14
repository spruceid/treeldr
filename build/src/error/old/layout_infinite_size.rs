use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutInfiniteSize {
	pub id: Id
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutInfiniteSize {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("the size of layout `{}` is infinite", self.id.with(vocab))
	}
}