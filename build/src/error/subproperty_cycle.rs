use treeldr::{
	vocab::TldrVocabulary
};
use locspan::{Span, MaybeLocated};

pub use crate::prop::SubPropertyCycle;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for SubPropertyCycle<M> {
	fn message(&self, _vocab: &TldrVocabulary) -> String {
		"sub-property cycle".to_string()
	}
}