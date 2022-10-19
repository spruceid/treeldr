use treeldr::{Feature, Vocabulary};
use locspan::{Span, MaybeLocated};

#[derive(Debug)]
pub struct UnimplementedFeature(pub Feature);

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for UnimplementedFeature {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("unimplemented feature `{}`", self.0)
	}
}