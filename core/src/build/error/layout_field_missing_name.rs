use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutFieldMissingName(pub Id);

impl<F> super::AnyError<F> for LayoutFieldMissingName {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no name defined for field `{}`", self.0.display(vocab))
	}
}