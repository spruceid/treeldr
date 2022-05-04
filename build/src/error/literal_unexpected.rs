use treeldr::{
	Vocabulary,
	vocab::Literal,
	vocab::Display
};

#[derive(Debug)]
pub struct LiteralUnexpected<F>(pub Literal<F>);

impl<F> super::AnyError<F> for LiteralUnexpected<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		format!("unexpected literal `{}`", self.0.display(vocab))
	}
}