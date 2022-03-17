use crate::{Id, node, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct NodeInvalidType<F> {
	pub id: Id,
	pub expected: node::Type,
	pub found: node::CausedTypes<F>,
}

impl<F> super::AnyError<F> for NodeInvalidType<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid type for {}", self.id.display(vocab))
	}
}