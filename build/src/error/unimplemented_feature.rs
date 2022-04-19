use treeldr::{Feature, Vocabulary};

#[derive(Debug)]
pub struct UnimplementedFeature(pub Feature);

impl<F> super::AnyError<F> for UnimplementedFeature {
	fn message(&self, _vocab: &Vocabulary) -> String {
		format!("unimplemented feature `{}`", self.0)
	}
}