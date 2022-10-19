use treeldr::{Id, Vocabulary, vocab::Display, layout::Primitive};
use locspan::{Span, MaybeLocated};
use crate::layout::primitive::Restriction;

#[derive(Debug)]
pub struct LayoutDatatypeRestrictionInvalid {
	pub id: Id,
	pub primitive: Primitive,
	pub restriction: Restriction
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutDatatypeRestrictionInvalid {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid datatype restriction of layout `{}`", self.id.display(vocab))
	}
}