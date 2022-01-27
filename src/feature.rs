use std::fmt;

/// Experimental/uncomplete features.
#[derive(Debug)]
pub enum Feature {}

impl Feature {
	fn name(&self) -> &'static str {
		unreachable!()
	}
}

impl fmt::Display for Feature {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.name().fmt(f)
	}
}
