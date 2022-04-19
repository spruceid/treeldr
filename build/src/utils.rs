pub mod scc;

pub use scc::SccGraph;

pub trait TryFromIterator<T>: Sized {
	fn try_from_iterator<E, I: IntoIterator<Item = Result<T, E>>>(iter: I) -> Result<Self, E>;

	fn try_from_filtered_iterator<E, I: IntoIterator<Item = Result<Option<T>, E>>>(
		iter: I,
	) -> Result<Self, E>;
}

impl<T> TryFromIterator<T> for Vec<T> {
	fn try_from_iterator<E, I: IntoIterator<Item = Result<T, E>>>(iter: I) -> Result<Self, E> {
		let mut result = Self::new();

		for item in iter {
			result.push(item?)
		}

		Ok(result)
	}

	fn try_from_filtered_iterator<E, I: IntoIterator<Item = Result<Option<T>, E>>>(
		iter: I,
	) -> Result<Self, E> {
		let mut result = Self::new();

		for item in iter {
			if let Some(item) = item? {
				result.push(item)
			}
		}

		Ok(result)
	}
}

pub trait TryCollect<T, E>: Iterator<Item = Result<T, E>> + Sized {
	fn try_collect<B: TryFromIterator<T>>(self) -> Result<B, E>;
}

pub trait TryFilterCollect<T, E>: Iterator<Item = Result<Option<T>, E>> + Sized {
	fn try_filter_collect<B: TryFromIterator<T>>(self) -> Result<B, E>;
}

impl<T, E, I: Iterator<Item = Result<T, E>>> TryCollect<T, E> for I {
	fn try_collect<B: TryFromIterator<T>>(self) -> Result<B, E> {
		B::try_from_iterator(self)
	}
}

impl<T, E, I: Iterator<Item = Result<Option<T>, E>>> TryFilterCollect<T, E> for I {
	fn try_filter_collect<B: TryFromIterator<T>>(self) -> Result<B, E> {
		B::try_from_filtered_iterator(self)
	}
}
