use treeldr::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct PropertyMissingType(pub Id);

impl<F> super::AnyError<F> for PropertyMissingType {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no range defined for property `{}`", self.0.display(vocab))
	}
}