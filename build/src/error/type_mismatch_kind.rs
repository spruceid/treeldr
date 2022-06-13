use treeldr::{Id, Vocabulary, ty::Kind};
use locspan::Location;

#[derive(Debug)]
pub struct TypeMismatchKind<F> {
	pub id: Id,
	pub expected: Option<Kind>,
	pub found: Option<Kind>,
	pub because: Option<Location<F>>
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
			Self::Restriction => "a restriction",
			Self::Enumeration => "an enumeration"
		}
	}
}

impl<F: Clone> super::AnyError<F> for TypeMismatchKind<F> {
	fn message(&self, _vocab: &Vocabulary) -> String {
		match (self.found, self.expected) {
			(Some(found), Some(expected)) => format!("type is not {} but {}", found.name(), expected.name()),
			(Some(found), None) => format!("type is not {}", found.name()),
			(None, Some(expected)) => format!("type is {}", expected.name()),
			(None, None) => "type definition mismatch".into()
		}
	}

	fn other_labels(&self, _vocab: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		let mut labels = Vec::new();
		if let Some(cause) = &self.because {
			let message = match self.expected {
				Some(expected) => format!("previously used as {} here", expected.name()),
				None => "previously defined here".into()
			};

			labels.push(cause.clone().into_secondary_label().with_message(message));
		}
		labels
	}
}