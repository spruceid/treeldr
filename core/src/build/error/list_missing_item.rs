use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct ListMissingItem(pub Id);

impl<F> super::AnyError<F> for ListMissingItem {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("missing item for list `{}`", self.0.display(vocab))
	}
}