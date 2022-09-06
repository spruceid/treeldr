use crate::{Id, node, Vocabulary, vocab::Display, reporting::MetadataDiagnostic};

#[derive(Debug)]
pub struct NodeUnknown {
	pub id: Id,
	pub expected_ty: Option<node::Type>
}

impl<M: MetadataDiagnostic> super::AnyError<M> for NodeUnknown {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("unknown node {}", self.id.display(vocab))
	}
}