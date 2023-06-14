use treeldr::{Id, node, IriIndex, BlankIdIndex};
use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;
use contextual::WithContext;

#[derive(Debug)]
pub struct NodeUnknown {
	pub id: Id,
	pub expected_ty: Option<node::Type>
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeUnknown {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("unknown node {}", self.id.with(vocab))
	}
}