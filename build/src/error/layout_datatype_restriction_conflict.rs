use locspan::{Span, MaybeLocated};
use treeldr::vocab::TldrVocabulary;

pub type LayoutDatatypeRestrictionConflict<M> = crate::layout::restriction::primitive::Conflict<M>;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LayoutDatatypeRestrictionConflict<M> where M::File: Clone {
	fn message(&self, _vocab: &TldrVocabulary) -> String {
		"conflicting restrictions".to_string()
	}

	fn primary_label(
			&self,
			_vocab: &TldrVocabulary,
		) -> Option<String> {
		Some("this restriction...".to_string())
	}

	fn other_labels(
			&self,
			_vocab: &TldrVocabulary,
		) -> Vec<codespan_reporting::diagnostic::Label<<M as MaybeLocated>::File>> {
		match self.1.metadata().optional_location() {
			Some(loc) => vec![
				loc.clone().into_secondary_label().with_message("...contradicts this restriction".to_string())
			],
			None => vec![]
		}
	}
}