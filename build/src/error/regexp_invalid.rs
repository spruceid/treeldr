use treeldr::{
	Vocabulary,
	ty::data::regexp::ParseError
};

#[derive(Debug)]
pub struct RegExpInvalid(pub String, pub ParseError);

impl<F> super::AnyError<F> for RegExpInvalid {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("invalid regular expression `{}`: {}", self.0, self.1)
	}
}