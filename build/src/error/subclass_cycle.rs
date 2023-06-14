use treeldr::vocab::TldrVocabulary;
use locspan::{Span, MaybeLocated};

pub use crate::ty::SubClassCycle;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for SubClassCycle<M> {
	fn message(&self, _vocab: &TldrVocabulary) -> String {
		"subclass cycle".to_string()
	}
}