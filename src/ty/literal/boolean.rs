use core::fmt;
use std::cmp::Ordering;

use crate::display::DisplaySignature;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Boolean {
	const_: Option<bool>,
}

impl Boolean {
	pub fn singleton(b: bool) -> Self {
		Self { const_: Some(b) }
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self.const_, other.const_) {
			(Some(a), Some(b)) => {
				if a == b {
					Some(Ordering::Equal)
				} else {
					None
				}
			}
			(None, Some(_)) => Some(Ordering::Greater),
			(Some(_), None) => Some(Ordering::Less),
			(None, None) => Some(Ordering::Equal),
		}
	}

	pub fn contains(&self, value: bool) -> bool {
		match self.const_ {
			Some(b) => b == value,
			None => true,
		}
	}
}

impl<R> DisplaySignature<R> for Boolean {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self.const_ {
			Some(true) => write!(f, "true"),
			Some(false) => write!(f, "false"),
			None => write!(f, "bool"),
		}
	}
}
