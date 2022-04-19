use treeldr::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutMissingDescription(pub Id);

impl<F> super::AnyError<F> for LayoutMissingDescription {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no implementation layout `{}`", self.0.display(vocab))
	}
}