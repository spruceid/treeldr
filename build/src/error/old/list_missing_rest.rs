use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct ListMissingRest(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for ListMissingRest {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("missing rest for list `{}`", self.0.with(vocab))
	}
}