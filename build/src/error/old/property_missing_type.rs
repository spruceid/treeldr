use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct PropertyMissingType(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for PropertyMissingType {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("no range defined for property `{}`", self.0.with(vocab))
	}
}