use treeldr::{
	IriIndex,
	BlankIdIndex, value::InvalidLiteral
};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};

pub type LiteralInvalid = InvalidLiteral;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralInvalid {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("invalid literal: {self}")
	}
}