use crate::{Id, Vocabulary, ty::Kind};
use locspan::Location;

#[derive(Debug)]
pub struct TypeMismatchKind<F> {
	pub id: Id,
	pub expected: Kind,
	pub found: Kind,
	pub because: Option<Location<F>>
}

trait KindName {
	fn name(&self) -> &str;
}

impl KindName for Kind {
	fn name(&self) -> &str {
		match self {
			Self::Normal => "a normal type",
			Self::Union => "an union"
		}
	}
}

impl<F: Clone> super::AnyError<F> for TypeMismatchKind<F> {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("type is not {} but {}", self.found.name(), self.expected.name())
	}

	fn other_labels(&self, _vocab: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		let mut labels = Vec::new();
		if let Some(cause) = &self.because {
			labels.push(cause.clone().into_secondary_label().with_message(format!("previously used as {} here", self.expected.name())));
		}
		labels
	}
}