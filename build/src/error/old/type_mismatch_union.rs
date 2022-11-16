use treeldr::{Id, IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct TypeMismatchUnion<M> {
	pub id: Id,
	pub expected: Id,
	pub found: Id,
	pub because: M
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for TypeMismatchUnion<M> where M::File: Clone {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("expected union list {}, found {}", self.expected.with(vocab), self.found.with(vocab))
	}

	fn other_labels(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		let mut labels = Vec::new();
		if let Some(cause) = self.because.optional_location().cloned() {
			labels.push(cause.into_secondary_label().with_message("union previously defined here".to_string()));
		}
		labels
	}
}