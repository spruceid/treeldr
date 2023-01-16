use treeldr::{
	IriIndex, BlankIdIndex
};
use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;

pub use crate::ty::SubClassCycle;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for SubClassCycle<M> {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		"subclass cycle".to_string()
	}
}