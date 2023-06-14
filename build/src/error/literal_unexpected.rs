use treeldr::{
	value::Literal, vocab::TldrVocabulary
};
use rdf_types::RdfDisplay;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LiteralUnexpected(pub Literal);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralUnexpected {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("unexpected literal `{}`", self.0.with(vocab).rdf_display())
	}
}