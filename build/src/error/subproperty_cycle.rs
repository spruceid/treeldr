use treeldr::{
	IriIndex, BlankIdIndex
};
use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;

pub use crate::prop::SubPropertyCycle;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for SubPropertyCycle<M> {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		"sub-property cycle".to_string()
	}
}