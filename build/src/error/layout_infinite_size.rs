use treeldr::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutInfiniteSize {
	pub id: Id
}

impl<F> super::AnyError<F> for LayoutInfiniteSize {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("the size of layout `{}` is infinite", self.id.display(vocab))
	}
}