use crate::{Id, node, Vocabulary, vocab::Display};

#[derive(Debug)]
pub struct NodeInvalidType<F> {
	pub id: Id,
	pub expected: node::Type,
	pub found: node::CausedTypes<F>,
}

impl node::Type {
	pub fn name(&self) -> &str {
		match self {
			node::Type::Type => "type",
			node::Type::Property => "property",
			node::Type::Layout => "layout",
			node::Type::LayoutField => "structure layout field",
			node::Type::LayoutVariant => "enum layout variant",
			node::Type::List => "list"
		}
	}
}

impl<F: Clone> super::AnyError<F> for NodeInvalidType<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("invalid type for {}", self.id.display(vocab))
	}

	fn other_labels(&self, _vocab: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		let mut labels = Vec::new();

		for ty in self.found.iter() {
			let (ty, cause) = ty.into_parts();
			if let Some(cause) = cause {
				labels.push(cause.into_secondary_label().with_message(format!("declared as a {} here", ty.name())));
			}
		}

		labels
	}

	fn notes(&self, _vocab: &Vocabulary) -> Vec<String> {
		let mut notes = Vec::new();

		notes.push(format!("expected a {}", self.expected.name()));

		for (i, ty) in self.found.iter().enumerate() {
			if i == 0 {
				notes.push(format!("found a {}", ty.name()))
			} else {
				notes.push(format!("      a {}", ty.name()))
			}
		}

		notes
	}
}