use crate::{Id, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct TypeIntersectionLiteralType(pub Id);

impl<F> super::AnyError<F> for TypeIntersectionLiteralType {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid literal value in type intersection `{}`", self.0.display(vocab))
	}
}