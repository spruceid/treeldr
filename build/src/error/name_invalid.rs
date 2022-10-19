use treeldr::{Vocabulary};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct NameInvalid(pub String);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NameInvalid {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("invalid name `{}`", self.0)
	}
}