use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LayoutMissingDatatypePrimitive(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMissingDatatypePrimitive {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("no base defined for datatype layout `{}`", self.0.display(vocab))
	}
}