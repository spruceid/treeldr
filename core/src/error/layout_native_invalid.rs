use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutNativeInvalid(pub Id);

impl<F> super::AnyError<F> for LayoutNativeInvalid {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid native layout `{}`", self.0.display(vocab))
	}
}