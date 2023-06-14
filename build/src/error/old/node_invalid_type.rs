use treeldr::{Id, node, IriIndex, BlankIdIndex};
use locspan::{Span, MaybeLocated, Meta};
use rdf_types::Vocabulary;
use contextual::WithContext;

#[derive(Debug)]
pub struct NodeInvalidType<M> {
	pub id: Id,
	pub expected: node::Type,
	pub found: node::TypesMetadata<M>,
}

trait NodeTypeName {
	fn name(&self) -> &str;
}

impl NodeTypeName for node::Type {
	fn name(&self) -> &str {
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

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeInvalidType<M> where M::File: Clone {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("invalid type for {}", self.id.with(vocab))
	}

	fn other_labels(&self, _vocab: &TldrVocabulary) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		let mut labels = Vec::new();

		for Meta(ty, ty_meta) in self.found.iter() {
			if let Some(loc) = ty_meta.optional_location().cloned() {
				labels.push(loc.into_secondary_label().with_message(format!("declared as a {} here", ty.name())));
			}
		}

		labels
	}

	fn notes(&self, _vocab: &TldrVocabulary) -> Vec<String> {
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