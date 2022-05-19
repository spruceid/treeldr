use treeldr::{Id, Vocabulary, vocab::Display, layout::Primitive};
use crate::layout::primitive::Restriction;

#[derive(Debug)]
pub struct LayoutDatatypeRestrictionInvalid {
	pub id: Id,
	pub primitive: Primitive,
	pub restriction: Restriction
}

impl<F> super::AnyError<F> for LayoutDatatypeRestrictionInvalid {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid datatype restriction of layout `{}`", self.id.display(vocab))
	}
}