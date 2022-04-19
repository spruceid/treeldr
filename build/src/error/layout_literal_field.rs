use treeldr::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutLiteralField(pub Id);

impl<F> super::AnyError<F> for LayoutLiteralField {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid literal field value in layout `{}`", self.0.display(vocab))
	}
}