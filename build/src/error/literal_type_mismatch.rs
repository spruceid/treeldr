use treeldr::{
	value::Literal, Id, vocab::TldrVocabulary
};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LiteralTypeMismatch {
	pub value: Literal,
	pub expected_type: Id
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralTypeMismatch {
	fn message(&self, _vocab: &TldrVocabulary) -> String {
		"invalid literal type".to_string()
	}
}