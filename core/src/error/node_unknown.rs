use crate::{Id, Type, vocab::TldrVocabulary};
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct NodeUnknown {
	pub id: Id,
	pub expected_ty: Type
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeUnknown {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("unknown node {}", self.id.with(vocab))
	}
}