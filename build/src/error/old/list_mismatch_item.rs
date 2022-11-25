use treeldr::{Id, vocab::Object, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct ListMismatchItem<M> {
	pub id: Id,
	pub expected: Object<M>,
	pub found: Object<M>,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for ListMismatchItem<M> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("item mismatch for list `{}`", self.id.with(vocab))
	}
}