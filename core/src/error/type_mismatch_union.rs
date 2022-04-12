use crate::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct TypeMismatchUnion<F> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: Option<Location<F>>
}

impl<F: Clone> super::AnyError<F> for TypeMismatchUnion<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("expected union list {}, found {}", self.expected.display(vocab), self.found.display(vocab))
	}

	fn other_labels(&self, _vocab: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		let mut labels = Vec::new();
		if let Some(cause) = &self.because {
			labels.push(cause.clone().into_secondary_label().with_message("union previously defined here".to_string()));
		}
		labels
	}
}