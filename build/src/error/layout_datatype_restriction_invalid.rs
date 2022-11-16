use treeldr::{Id, IriIndex, BlankIdIndex, layout::Primitive};
use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;
use contextual::WithContext;
use crate::layout::restriction::primitive::Restriction;

#[derive(Debug)]
pub struct LayoutDatatypeRestrictionInvalid {
	pub id: Id,
	pub primitive: Primitive,
	pub restriction: Restriction
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutDatatypeRestrictionInvalid {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("invalid datatype restriction of layout `{}`", self.id.with(vocab))
	}
}