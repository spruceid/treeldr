use educe::Educe;

use crate::{DatasetPattern, TypeRef};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe)]
#[educe(Default)]
pub struct Bound<R, T = TypeRef> {
	/// Introduced variables.
	pub intro: Vec<T>,

	/// Dataset constraints.
	pub dataset: DatasetPattern<R>,
}

impl<R, T> Bound<R, T> {
	pub fn map_types<U>(self, mut f: impl FnMut(T) -> U) -> Bound<R, U> {
		Bound {
			intro: self.intro.into_iter().map(|x| f(x)).collect(),
			dataset: self.dataset
		}
	}
}