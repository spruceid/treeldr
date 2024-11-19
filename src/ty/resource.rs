use core::fmt;
use std::{cmp::Ordering, collections::BTreeSet};

use educe::Educe;

use crate::display::DisplaySignature;

/// Resource type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe)]
#[educe(Default)]
pub struct Resource<R> {
	set: Option<BTreeSet<R>>,
}

impl<R: Ord> Resource<R> {
	pub fn singleton(r: R) -> Self {
		let mut set = BTreeSet::new();
		set.insert(r);

		Self { set: Some(set) }
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (&self.set, &other.set) {
			(Some(a), Some(b)) => {
				if a == b {
					Some(Ordering::Equal)
				} else if a.is_superset(b) {
					Some(Ordering::Greater)
				} else if a.is_subset(b) {
					Some(Ordering::Less)
				} else {
					None
				}
			}
			(None, Some(_)) => Some(Ordering::Greater),
			(Some(_), None) => Some(Ordering::Less),
			(None, None) => Some(Ordering::Equal),
		}
	}

	pub fn contains(&self, resource: &R) -> bool {
		match &self.set {
			Some(set) => set.contains(resource),
			None => true,
		}
	}
}

impl<R: fmt::Display> DisplaySignature<R> for Resource<R> {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match &self.set {
			Some(set) => {
				write!(f, "(any_of")?;

				for r in set {
					write!(f, " {r}")?;
				}

				write!(f, ")")
			}
			None => {
				write!(f, "resource")
			}
		}
	}
}
