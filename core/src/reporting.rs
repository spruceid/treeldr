use crate::{vocab, Vocabulary};
use locspan::{Loc, Location};

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

/// Error with diagnostic reporting support.
pub trait DiagnoseWithCause<F> {
	fn message(&self, cause: Option<&Location<F>>) -> String;

	fn labels(
		&self,
		_cause: Option<&Location<F>>,
	) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		Vec::new()
	}

	fn notes(&self, _cause: Option<&Location<F>>) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(
		&self,
		cause: Option<&Location<F>>,
	) -> codespan_reporting::diagnostic::Diagnostic<F> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message(cause))
			.with_labels(self.labels(cause))
			.with_notes(self.notes(cause))
	}
}

/// Error with diagnostic reporting support.
pub trait DiagnoseWithCauseAndVocabulary<F> {
	fn message(&self, cause: Option<&Location<F>>, vocabulary: &Vocabulary) -> String;

	fn labels(
		&self,
		_cause: Option<&Location<F>>,
		_vocabulary: &Vocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		Vec::new()
	}

	fn notes(&self, _cause: Option<&Location<F>>, _vocabulary: &Vocabulary) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(
		&self,
		cause: Option<&Location<F>>,
		vocabulary: &Vocabulary,
	) -> codespan_reporting::diagnostic::Diagnostic<F> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message(cause, vocabulary))
			.with_labels(self.labels(cause, vocabulary))
			.with_notes(self.notes(cause, vocabulary))
	}
}

/// Error with diagnostic reporting support.
pub trait DiagnoseWithVocabulary<F> {
	fn message(&self, vocabulary: &Vocabulary) -> String;

	fn labels(&self, _vocabulary: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		Vec::new()
	}

	fn notes(&self, _vocabulary: &Vocabulary) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(&self, vocabulary: &Vocabulary) -> codespan_reporting::diagnostic::Diagnostic<F> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message(vocabulary))
			.with_labels(self.labels(vocabulary))
			.with_notes(self.notes(vocabulary))
	}
}

impl<'t, 'v, F, T: DiagnoseWithVocabulary<F>> Diagnose<F> for vocab::WithVocabulary<'t, 'v, T> {
	fn message(&self) -> String {
		self.value().message(self.vocabulary())
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		self.value().labels(self.vocabulary())
	}

	fn notes(&self) -> Vec<String> {
		self.value().notes(self.vocabulary())
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<F> {
		self.value().diagnostic(self.vocabulary())
	}
}

impl<F, T: DiagnoseWithCause<F>> Diagnose<F> for crate::Caused<T, F> {
	fn message(&self) -> String {
		self.inner().message(self.cause())
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		self.inner().labels(self.cause())
	}

	fn notes(&self) -> Vec<String> {
		self.inner().notes(self.cause())
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<F> {
		self.inner().diagnostic(self.cause())
	}
}

impl<F, T: DiagnoseWithCauseAndVocabulary<F>> DiagnoseWithVocabulary<F> for crate::Caused<T, F> {
	fn message(&self, vocabulary: &Vocabulary) -> String {
		self.inner().message(self.cause(), vocabulary)
	}

	fn labels(&self, vocabulary: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		self.inner().labels(self.cause(), vocabulary)
	}

	fn notes(&self, vocabulary: &Vocabulary) -> Vec<String> {
		self.inner().notes(self.cause(), vocabulary)
	}

	fn diagnostic(&self, vocabulary: &Vocabulary) -> codespan_reporting::diagnostic::Diagnostic<F> {
		self.inner().diagnostic(self.cause(), vocabulary)
	}
}

impl<F, T: DiagnoseWithCause<F>> Diagnose<F> for Loc<T, F> {
	fn message(&self) -> String {
		self.value().message(Some(self.location()))
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		self.value().labels(Some(self.location()))
	}

	fn notes(&self) -> Vec<String> {
		self.value().notes(Some(self.location()))
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<F> {
		self.value().diagnostic(Some(self.location()))
	}
}

impl<F, T: DiagnoseWithCauseAndVocabulary<F>> DiagnoseWithVocabulary<F> for Loc<T, F> {
	fn message(&self, vocabulary: &Vocabulary) -> String {
		self.value().message(Some(self.location()), vocabulary)
	}

	fn labels(&self, vocabulary: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		self.value().labels(Some(self.location()), vocabulary)
	}

	fn notes(&self, vocabulary: &Vocabulary) -> Vec<String> {
		self.value().notes(Some(self.location()), vocabulary)
	}

	fn diagnostic(&self, vocabulary: &Vocabulary) -> codespan_reporting::diagnostic::Diagnostic<F> {
		self.value().diagnostic(Some(self.location()), vocabulary)
	}
}
