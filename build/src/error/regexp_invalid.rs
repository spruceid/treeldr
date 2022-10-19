use treeldr::{
	Vocabulary,
	ty::data::regexp::ParseError
};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct RegExpInvalid(pub String, pub ParseError);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for RegExpInvalid {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("invalid regular expression `{}`: {}", self.0, self.1)
	}
}