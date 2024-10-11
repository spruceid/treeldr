use std::{cmp::Ordering, hash::Hash, ops::{Deref, DerefMut}};

#[derive(Debug, Default, Clone, Copy)]
pub struct Ignored<T>(pub T);

impl<T> PartialEq for Ignored<T> {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl<T> Eq for Ignored<T> {}

impl<T> PartialOrd for Ignored<T> {
	fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
		Some(Ordering::Equal)
	}
}

impl<T> Ord for Ignored<T> {
	fn cmp(&self, _other: &Self) -> Ordering {
		Ordering::Equal
	}
}

impl<T> Hash for Ignored<T> {
	fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl<T> Deref for Ignored<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for Ignored<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}