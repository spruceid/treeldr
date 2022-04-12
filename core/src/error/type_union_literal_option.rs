use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct TypeUnionLiteralOption(pub Id);

impl<F> super::AnyError<F> for TypeUnionLiteralOption {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid literal option value in type union `{}`", self.0.display(vocab))
	}
}