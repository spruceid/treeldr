use treeldr::{Id, Type, vocab::TldrVocabulary};
use locspan::{Span, MaybeLocated, Meta};
use contextual::WithContext;

use crate::Error;

#[derive(Debug)]
pub struct NodeUnknown {
	pub id: Id,
	pub expected_type: Option<Type>
}

impl NodeUnknown {
	pub fn at<M>(self, meta: M) -> Error<M> {
		Meta(self.into(), meta)
	}
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeUnknown {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("unknown node {}", self.id.with(vocab))
	}
}