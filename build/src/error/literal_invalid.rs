use treeldr::{
	value::InvalidLiteral, vocab::TldrVocabulary
};
use locspan::{Span, MaybeLocated};

pub type LiteralInvalid = InvalidLiteral;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralInvalid {
	fn message(&self, _vocab: &TldrVocabulary) -> String {
		format!("invalid literal: {self}")
	}
}