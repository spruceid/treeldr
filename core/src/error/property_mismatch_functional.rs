use crate::{Id, Vocabulary, vocab::Display};
use locspan::Location;

#[derive(Debug)]
pub struct PropertyMismatchFunctional<F> {
	pub id: Id,
	pub expected: bool,
	pub found: bool,
	pub because: Option<Location<F>>
}

impl<F> super::AnyError<F> for PropertyMismatchFunctional<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("functional status mismatch for property `{}`", self.id.display(vocab))
	}
}