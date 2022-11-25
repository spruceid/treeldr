use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutMissingDatatypePrimitive(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMissingDatatypePrimitive {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("no base defined for datatype layout `{}`", self.0.with(vocab))
	}
}