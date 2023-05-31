use crate::PropertyValue;

use super::RestrictionSet;
use locspan::Meta;

pub mod float;
pub mod integer;
pub mod none;
pub mod string;
pub mod unicode_string;

use super::Restrictions;

pub trait RestrictionsTemplate<T> {
	type Ref<'a>
	where
		T: 'a;

	type Set<M>: RestrictionSet;

	type Iter<'a, M>
	where
		T: 'a,
		M: 'a;
}

/// Values of the `tldr:withRestrictions` property.
pub struct WithRestrictions<'a, M> {
	pub(crate) restrictions: Meta<Restrictions<'a, M>, &'a M>,
}

impl<'a, M> WithRestrictions<'a, M> {
	pub(crate) fn new(restrictions: Meta<Restrictions<'a, M>, &'a M>) -> Option<Self> {
		if restrictions.is_restricted() {
			Some(Self { restrictions })
		} else {
			None
		}
	}

	pub fn iter(&self) -> WithRestrictionsIter<'a, M> {
		WithRestrictionsIter {
			restrictions: Some(self.restrictions),
		}
	}
}

/// Iterator over the values of the `tldr:withRestrictions` property.
pub struct WithRestrictionsIter<'a, M> {
	restrictions: Option<Meta<Restrictions<'a, M>, &'a M>>,
}

impl<'a, M> Iterator for WithRestrictionsIter<'a, M> {
	type Item = PropertyValue<Restrictions<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.restrictions
			.take()
			.map(|r| PropertyValue::new(None, r))
	}
}
