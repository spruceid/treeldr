use treeldr::{
	vocab::Literal,
	IriIndex,
	BlankIdIndex
};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};
use contextual::WithContext;

#[derive(Debug)]
pub struct LiteralUnexpected<M>(pub Literal<M>);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralUnexpected<M> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("unexpected literal `{}`", self.0.with(vocab))
	}
}