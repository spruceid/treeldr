use std::fmt;
use std::path::PathBuf;
use treeldr_load as load;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
	File(PathBuf),
	Xsd,
}

impl<P: AsRef<std::path::Path>> From<P> for Source {
	fn from(path: P) -> Self {
		Self::File(path.as_ref().into())
	}
}

impl<'a> load::DisplayPath<'a> for Source {
	type Display = Display<'a>;

	fn display(&'a self) -> Self::Display {
		match self {
			Self::File(path) => Display::File(path.display()),
			Self::Xsd => Display::Xsd,
		}
	}
}

pub enum Display<'a> {
	File(std::path::Display<'a>),
	Xsd,
}

impl<'a> fmt::Display for Display<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::File(path) => path.fmt(f),
			Self::Xsd => write!(f, "http://www.w3.org/2001/XMLSchema"),
		}
	}
}
