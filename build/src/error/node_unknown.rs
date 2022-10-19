use treeldr::{Id, node, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct NodeUnknown {
	pub id: Id,
	pub expected_ty: Option<node::Type>
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeUnknown {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("unknown node {}", self.id.display(vocab))
	}
}