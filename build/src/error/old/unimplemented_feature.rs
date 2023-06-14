use rdf_types::Vocabulary;
use treeldr::{Feature, IriIndex, BlankIdIndex};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct UnimplementedFeature(pub Feature);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for UnimplementedFeature {
	fn message(&self, _vocab: &TldrVocabulary) -> String {
		format!("unimplemented feature `{}`", self.0)
	}
}