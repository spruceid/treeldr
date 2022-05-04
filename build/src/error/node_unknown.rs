use treeldr::{Id, node, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct NodeUnknown {
	pub id: Id,
	pub expected_ty: Option<node::Type>
}

impl<F> super::AnyError<F> for NodeUnknown {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("unknown node {}", self.id.display(vocab))
	}
}