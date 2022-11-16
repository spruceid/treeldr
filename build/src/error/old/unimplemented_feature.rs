use rdf_types::Vocabulary;
use treeldr::{Feature, IriIndex, BlankIdIndex};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct UnimplementedFeature(pub Feature);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for UnimplementedFeature {
	fn message(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("unimplemented feature `{}`", self.0)
	}
}