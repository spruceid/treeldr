use std::fmt;

/// Experimental/uncomplete features.
#[derive(Debug)]
pub enum Feature {
	Error(&'static str),
}

impl Feature {
	fn name(&self) -> String {
		match self {
			Self::Error(s) => format!("error `{}`", s),
		}
	}
}

impl fmt::Display for Feature {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.name().fmt(f)
	}
}
