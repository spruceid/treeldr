use treeldr::Vocabulary;
use rdf_types::loc::Literal;

#[derive(Debug)]
pub struct LiteralUnexpected<F>(pub Literal<F>);

impl<F> super::AnyError<F> for LiteralUnexpected<F> {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("unexpected literal `{}`", self.0)
	}
}