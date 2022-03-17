use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutFieldMissingProperty(pub Id);

impl<F> super::AnyError<F> for LayoutFieldMissingProperty {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no property defined for field `{}`", self.0.display(vocab))
	}
}