use educe::Educe;

use crate::{DatasetPattern, TypeRef};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(Default)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct Bound<R, T = TypeRef<R>> {
	/// Introduced variables.
	pub intro: Vec<T>,

	/// Dataset constraints.
	pub dataset: DatasetPattern<R>,
}
