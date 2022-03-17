/// Error with diagnostic reporting support.
pub trait Diagnose<F> {
	fn message(&self) -> String;

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		Vec::new()
	}

	fn notes(&self) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<F> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message())
			.with_labels(self.labels())
			.with_notes(self.notes())
	}
}
