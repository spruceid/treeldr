use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutFieldMissingLayout(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutFieldMissingLayout {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("no layout defined for field `{}`", self.0.with(vocab))
	}
}