use treeldr::{Id, IriIndex, BlankIdIndex, Type};
use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;
use contextual::WithContext;

#[derive(Debug)]
pub struct NodeUnknown {
	pub id: Id,
	pub expected_type: Option<Type>
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeUnknown {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("unknown node {}", self.id.with(vocab))
	}
}