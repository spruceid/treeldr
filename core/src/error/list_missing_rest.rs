use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct ListMissingRest(pub Id);

impl<F> super::AnyError<F> for ListMissingRest {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("missing rest for list `{}`", self.0.display(vocab))
	}
}