use treeldr::{Id, vocab::TldrVocabulary};
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

use crate::layout::SingleDescriptionProperty;

#[derive(Debug)]
pub struct LayoutDescriptionMismatch<M> {
	pub id: Id,
	pub desc1: SingleDescriptionProperty<M>,
	pub desc2: SingleDescriptionProperty<M>
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutDescriptionMismatch<M> {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("implementation mismatch for layout `{}`", self.id.with(vocab))
	}
}