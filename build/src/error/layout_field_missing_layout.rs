use treeldr::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutFieldMissingLayout(pub Id);

impl<F> super::AnyError<F> for LayoutFieldMissingLayout {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no layout defined for field `{}`", self.0.display(vocab))
	}
}