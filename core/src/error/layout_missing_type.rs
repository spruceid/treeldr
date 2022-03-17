use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutMissingType(pub Id);

impl<F> super::AnyError<F> for LayoutMissingType {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no type defined for layout `{}`", self.0.display(vocab))
	}
}