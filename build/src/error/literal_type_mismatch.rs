use treeldr::{
	IriIndex,
	BlankIdIndex, value::Literal, Id
};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LiteralTypeMismatch {
	pub value: Literal,
	pub expected_type: Id
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralTypeMismatch {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		"invalid literal type".to_string()
	}
}