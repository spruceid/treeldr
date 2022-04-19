use treeldr::{
	Vocabulary,
	layout::literal::regexp::ParseError
};

#[derive(Debug)]
pub struct RegExpInvalid(pub rdf_types::StringLiteral, pub ParseError);

impl<F> super::AnyError<F> for RegExpInvalid {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("invalid regular expression {}: {}", self.0, self.1)
	}
}