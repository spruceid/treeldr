use treeldr::{
	IriIndex, BlankIdIndex
};
use locspan::{Span, MaybeLocated};
use rdf_types::Vocabulary;

pub use crate::ty::TypeCycle;

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for TypeCycle<M> {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("type cycle")
	}
}