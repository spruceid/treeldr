use crate::Vocabulary;

#[derive(Debug)]
pub struct RegExpInvalid(pub rdf_types::StringLiteral, pub crate::layout::literal::regexp::ParseError);

impl<F> super::AnyError<F> for RegExpInvalid {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("invalid regular expression {}: {}", self.0, self.1)
	}
}