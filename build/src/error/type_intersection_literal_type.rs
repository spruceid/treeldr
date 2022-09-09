use treeldr::{Id, Vocabulary, vocab::Display};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct TypeIntersectionLiteralType(pub Id);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for TypeIntersectionLiteralType {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid literal value in type intersection `{}`", self.0.display(vocab))
	}
}