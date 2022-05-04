use treeldr::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutIntersectionFailed {
	pub id: Id
}

impl<F> super::AnyError<F> for LayoutIntersectionFailed {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("intersection `{}` failed", self.id.display(vocab))
	}
}