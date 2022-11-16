use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutDescriptionMissing(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutDescriptionMissing {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("no implementation for layout `{}`", self.0.with(vocab))
	}
}