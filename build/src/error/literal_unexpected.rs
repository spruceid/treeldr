use treeldr::{
	Vocabulary,
	vocab::Literal,
	vocab::Display
};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct LiteralUnexpected<M>(pub Literal<M>);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for LiteralUnexpected<M> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("unexpected literal `{}`", self.0.display(vocab))
	}
}