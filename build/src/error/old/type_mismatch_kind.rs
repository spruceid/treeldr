use treeldr::{Id, ty::Kind, IriIndex, BlankIdIndex};
use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;

#[derive(Debug)]
pub struct TypeMismatchKind<M> {
	pub id: Id,
	pub expected: Option<Kind>,
	pub found: Option<Kind>,
	pub because: M
}

trait KindName {
	fn name(&self) -> &str;
}

impl KindName for Kind {
	fn name(&self) -> &str {
		match self {
			Self::Empty => "the empty type",
			Self::Data => "a datatype",
			Self::Normal => "a normal type",
			Self::Union => "an union",
			Self::Intersection => "an intersection",
			Self::Restriction => "a restriction"
		}
	}
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for TypeMismatchKind<M> where M::File: Clone {
	fn message(&self, _vocab: &TldrVocabulary) -> String {
		match (self.found, self.expected) {
			(Some(found), Some(expected)) => format!("type is not {} but {}", found.name(), expected.name()),
			(Some(found), None) => format!("type is not {}", found.name()),
			(None, Some(expected)) => format!("type is {}", expected.name()),
			(None, None) => "type definition mismatch".into()
		}
	}

	fn other_labels(&self, _vocab: &TldrVocabulary) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		let mut labels = Vec::new();
		if let Some(cause) = self.because.optional_location().cloned() {
			let message = match self.expected {
				Some(expected) => format!("previously used as {} here", expected.name()),
				None => "previously defined here".into()
			};

			labels.push(cause.into_secondary_label().with_message(message));
		}
		labels
	}
}