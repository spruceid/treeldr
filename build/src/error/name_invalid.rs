use treeldr::vocab::TldrVocabulary;
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct NameInvalid(pub String);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NameInvalid {
	fn message(&self, _vocab: &TldrVocabulary) -> String {
		format!("invalid name `{}`", self.0)
	}
}