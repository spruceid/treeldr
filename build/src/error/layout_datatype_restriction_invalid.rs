use treeldr::{Id, layout::Primitive, vocab::TldrVocabulary};
use locspan::{Span, MaybeLocated};
use contextual::WithContext;
use crate::layout::restriction::primitive::Restriction;

#[derive(Debug)]
pub struct LayoutDatatypeRestrictionInvalid {
	pub id: Id,
	pub primitive: Primitive,
	pub restriction: Restriction
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutDatatypeRestrictionInvalid {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("invalid datatype restriction of layout `{}`", self.id.with(vocab))
	}
}