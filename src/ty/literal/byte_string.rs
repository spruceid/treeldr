use core::fmt;
use std::cmp::Ordering;

use crate::display::DisplaySignature;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteString;

impl ByteString {
	pub fn subtype_cmp(&self, _other: &Self) -> Option<Ordering> {
		Some(Ordering::Equal)
	}

	pub fn contains(&self, _value: &[u8]) -> bool {
		true
	}
}

impl<P, R> DisplaySignature<P, R> for ByteString {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<
			impl crate::Registry<PackageId = P, Resource = R>,
		>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		write!(f, "bytes")
	}
}
