use treeldr::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutLiteralIntersection(pub Id);

impl<F> super::AnyError<F> for LayoutLiteralIntersection {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid literal intersection component value in layout `{}`", self.0.display(vocab))
	}
}