use treeldr::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct LayoutMissingDatatypePrimitive(pub Id);

impl<F> super::AnyError<F> for LayoutMissingDatatypePrimitive {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no base defined for datatype layout `{}`", self.0.display(vocab))
	}
}