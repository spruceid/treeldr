use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LayoutMismatchPrimitive<M> {
	pub id: Id,
	pub expected: treeldr::layout::Primitive,
	pub found: treeldr::layout::Primitive,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutMismatchPrimitive<M> {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("datatype primitive mismatch for layout `{}`", self.id.with(vocab))
	}
}