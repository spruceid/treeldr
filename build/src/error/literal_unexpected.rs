use treeldr::{
	IriIndex,
	BlankIdIndex, value::Literal
};
use rdf_types::{Vocabulary, RdfDisplay, vocabulary::LanguageTagIndex};
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LiteralUnexpected(pub Literal);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralUnexpected {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex, LanguageTag = LanguageTagIndex>) -> String {
		format!("unexpected literal `{}`", self.0.with(vocab).rdf_display())
	}
}