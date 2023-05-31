use std::{cmp::Ordering, hash::Hash, marker::PhantomData};

use derivative::Derivative;
use locspan::{Meta, StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};

use crate::layout::primitive::RestrictionSet;

use super::RestrictionsTemplate;

pub struct Template;

impl<T> RestrictionsTemplate<T> for Template {
	type Ref<'a> = RestrictionRef<'a, T> where T: 'a;
	type Set<M> = Restrictions<T, M>;
	type Iter<'a, M> = Iter<'a, T, M> where T: 'a, M: 'a;
}

#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct Restriction<T>(PhantomData<T>);

#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct RestrictionRef<'a, T>(PhantomData<&'a T>);

#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct Restrictions<T, M>(PhantomData<(T, M)>);

impl<T: PartialEq, M> StrippedPartialEq for Restrictions<T, M> {
	fn stripped_eq(&self, _other: &Self) -> bool {
		true
	}
}

impl<T: Eq, M> StrippedEq for Restrictions<T, M> {}

impl<T: PartialOrd, M> StrippedPartialOrd for Restrictions<T, M> {
	fn stripped_partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
		Some(Ordering::Equal)
	}
}

impl<T: Ord, M> StrippedOrd for Restrictions<T, M> {
	fn stripped_cmp(&self, _other: &Self) -> Ordering {
		Ordering::Equal
	}
}

impl<T: Hash, M> StrippedHash for Restrictions<T, M> {
	fn stripped_hash<H: std::hash::Hasher>(&self, _state: &mut H) {
		// nothing to hash
	}
}

#[derive(Debug)]
pub struct Conflict<T, M>(pub Restriction<T>, pub Meta<Restriction<T>, M>);

impl<T, M> Default for Restrictions<T, M> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<T, M> Restrictions<T, M> {
	pub fn is_restricted(&self) -> bool {
		false
	}

	pub fn iter(&self) -> Iter<T, M> {
		Iter(PhantomData)
	}
}

impl<T, M> RestrictionSet for Restrictions<T, M> {
	fn is_restricted(&self) -> bool {
		self.is_restricted()
	}
}

pub struct Iter<'a, T, M>(PhantomData<&'a (T, M)>);

impl<'a, T, M> Iterator for Iter<'a, T, M> {
	type Item = Meta<RestrictionRef<'a, T>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		None
	}
}

impl<'a, T, M> DoubleEndedIterator for Iter<'a, T, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		None
	}
}
