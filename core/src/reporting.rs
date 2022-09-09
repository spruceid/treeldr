use crate::{vocab, Vocabulary};
use locspan::{Meta, MaybeLocated};

/// Error with diagnostic reporting support.
pub trait Diagnose<M: MaybeLocated> {
	fn message(&self) -> String;

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(&self) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message())
			.with_labels(self.labels())
			.with_notes(self.notes())
	}
}

/// Error with diagnostic reporting support.
pub trait DiagnoseWithMetadata<M: MaybeLocated> {
	fn message(&self, metadata: &M) -> String;

	fn labels(&self, _metadata: &M) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(&self, _metadata: &M) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(&self, metadata: &M) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message(metadata))
			.with_labels(self.labels(metadata))
			.with_notes(self.notes(metadata))
	}
}

/// Error with diagnostic reporting support.
pub trait DiagnoseWithMetadataAndVocabulary<M: MaybeLocated> {
	fn message(&self, metadata: &M, vocabulary: &Vocabulary) -> String;

	fn labels(
		&self,
		_metadata: &M,
		_vocabulary: &Vocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(&self, _metadata: &M, _vocabulary: &Vocabulary) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(
		&self,
		metadata: &M,
		vocabulary: &Vocabulary,
	) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message(metadata, vocabulary))
			.with_labels(self.labels(metadata, vocabulary))
			.with_notes(self.notes(metadata, vocabulary))
	}
}

/// Error with diagnostic reporting support.
pub trait DiagnoseWithVocabulary<M: MaybeLocated> {
	fn message(&self, vocabulary: &Vocabulary) -> String;

	fn labels(
		&self,
		_vocabulary: &Vocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(&self, _vocabulary: &Vocabulary) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(
		&self,
		vocabulary: &Vocabulary,
	) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message(vocabulary))
			.with_labels(self.labels(vocabulary))
			.with_notes(self.notes(vocabulary))
	}
}

impl<'t, 'v, M: MaybeLocated, T: DiagnoseWithVocabulary<M>> Diagnose<M>
	for vocab::WithVocabulary<'t, 'v, T>
{
	fn message(&self) -> String {
		self.value().message(self.vocabulary())
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		self.value().labels(self.vocabulary())
	}

	fn notes(&self) -> Vec<String> {
		self.value().notes(self.vocabulary())
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		self.value().diagnostic(self.vocabulary())
	}
}

impl<M: MaybeLocated, T: DiagnoseWithMetadata<M>> Diagnose<M> for Meta<T, M> {
	fn message(&self) -> String {
		self.value().message(self.metadata())
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		self.value().labels(self.metadata())
	}

	fn notes(&self) -> Vec<String> {
		self.value().notes(self.metadata())
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		self.value().diagnostic(self.metadata())
	}
}

impl<M: MaybeLocated, T: DiagnoseWithMetadataAndVocabulary<M>> DiagnoseWithVocabulary<M>
	for Meta<T, M>
{
	fn message(&self, vocabulary: &Vocabulary) -> String {
		self.value().message(self.metadata(), vocabulary)
	}

	fn labels(
		&self,
		vocabulary: &Vocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		self.value().labels(self.metadata(), vocabulary)
	}

	fn notes(&self, vocabulary: &Vocabulary) -> Vec<String> {
		self.value().notes(self.metadata(), vocabulary)
	}

	fn diagnostic(
		&self,
		vocabulary: &Vocabulary,
	) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		self.value().diagnostic(self.metadata(), vocabulary)
	}
}
