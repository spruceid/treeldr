use treeldr::{
	ty::data::regexp::ParseError,
	IriIndex, BlankIdIndex
};
use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;

#[derive(Debug)]
pub struct RegExpInvalid(pub String, pub ParseError);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for RegExpInvalid {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("invalid regular expression `{}`: {}", self.0, self.1)
	}
}