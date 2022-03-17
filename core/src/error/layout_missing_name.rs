use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutMissingName(pub Id);

impl<F> super::AnyError<F> for LayoutMissingName {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no name defined for layout `{}`", self.0.display(vocab))
	}
}