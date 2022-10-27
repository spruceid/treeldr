use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;
use treeldr::{IriIndex, BlankIdIndex};

pub type LayoutContainerRestrictionConflict<M> = treeldr::layout::container::restriction::Conflict<M>;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutContainerRestrictionConflict<M> where M::File: Clone {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		match self {
			Self::Cardinal(c) => c.message(vocab)
		}
	}

	fn primary_label(
			&self,
			vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		) -> Option<String> {
		match self {
			Self::Cardinal(c) => c.primary_label(vocab)
		}
	}

	fn other_labels(
			&self,
			vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		) -> Vec<codespan_reporting::diagnostic::Label<<M as MaybeLocated>::File>> {
		match self {
			Self::Cardinal(c) => c.other_labels(vocab)
		}
	}
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for treeldr::layout::container::restriction::cardinal::Conflict<M> where M::File: Clone {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		"conflicting restrictions".to_string()
	}

	fn primary_label(
			&self,
			_vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		) -> Option<String> {
		Some("this restriction...".to_string())
	}

	fn other_labels(
			&self,
			_vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		) -> Vec<codespan_reporting::diagnostic::Label<<M as MaybeLocated>::File>> {
			match self.1.metadata().optional_location() {
				Some(loc) => vec![
					loc.clone().into_secondary_label().with_message("...contradicts this restriction".to_string())
				],
				None => vec![]
			}
	}
}