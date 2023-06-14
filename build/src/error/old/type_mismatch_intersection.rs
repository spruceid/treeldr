use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct TypeMismatchIntersection<M> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for TypeMismatchIntersection<M> where M::File: Clone {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("expected intersection list {}, found {}", self.expected.with(vocab), self.found.with(vocab))
	}

	fn other_labels(&self, _vocab: &TldrVocabulary) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		let mut labels = Vec::new();
		if let Some(loc) = self.because.optional_location().cloned() {
			labels.push(loc.into_secondary_label().with_message("intersection previously defined here".to_string()));
		}
		labels
	}
}