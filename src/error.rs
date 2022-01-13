use crate::source;

/// Error with diagnostic reporting support.
pub trait Error: std::fmt::Display {
	fn message(&self) -> String;

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<source::Id>> {
		Vec::new()
	}

	fn notes(&self) -> Vec<String> {
		Vec::new()
	}
}