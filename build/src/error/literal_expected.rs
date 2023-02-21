use treeldr::{
	IriIndex,
	BlankIdIndex, Id
};
use rdf_types::{Vocabulary, RdfDisplay};
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LiteralExpected(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralExpected {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("expected literal value, found `{}`", self.0.with(vocab).rdf_display())
	}
}