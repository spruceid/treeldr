use treeldr::{IriIndex, BlankIdIndex};
use rdf_types::Vocabulary;
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct NameInvalid(pub String);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NameInvalid {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("invalid name `{}`", self.0)
	}
}