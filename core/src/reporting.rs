use crate::vocab::TldrVocabulary;
use contextual::Contextual;
use locspan::{MaybeLocated, Meta};

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
	fn message(&self, metadata: &M, vocabulary: &TldrVocabulary) -> String;

	fn labels(
		&self,
		_metadata: &M,
		_vocabulary: &TldrVocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(&self, _metadata: &M, _vocabulary: &TldrVocabulary) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(
		&self,
		metadata: &M,
		vocabulary: &TldrVocabulary,
	) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message(metadata, vocabulary))
			.with_labels(self.labels(metadata, vocabulary))
			.with_notes(self.notes(metadata, vocabulary))
	}
}

impl<M: MaybeLocated, T: DiagnoseWithMetadataAndVocabulary<M>> DiagnoseWithMetadataAndVocabulary<M>
	for Box<T>
{
	fn message(&self, metadata: &M, vocabulary: &TldrVocabulary) -> String {
		T::message(self, metadata, vocabulary)
	}

	fn labels(
		&self,
		metadata: &M,
		vocabulary: &TldrVocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		T::labels(self, metadata, vocabulary)
	}

	fn notes(&self, metadata: &M, vocabulary: &TldrVocabulary) -> Vec<String> {
		T::notes(self, metadata, vocabulary)
	}

	fn diagnostic(
		&self,
		metadata: &M,
		vocabulary: &TldrVocabulary,
	) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		T::diagnostic(self, metadata, vocabulary)
	}
}

/// Error with diagnostic reporting support.
pub trait DiagnoseWithVocabulary<M: MaybeLocated> {
	fn message(&self, vocabulary: &TldrVocabulary) -> String;

	fn labels(
		&self,
		_vocabulary: &TldrVocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(&self, _vocabulary: &TldrVocabulary) -> Vec<String> {
		Vec::new()
	}

	fn diagnostic(
		&self,
		vocabulary: &TldrVocabulary,
	) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		codespan_reporting::diagnostic::Diagnostic::error()
			.with_message(self.message(vocabulary))
			.with_labels(self.labels(vocabulary))
			.with_notes(self.notes(vocabulary))
	}
}

impl<M: MaybeLocated, T: DiagnoseWithMetadata<M>> DiagnoseWithMetadata<M> for Box<T> {
	fn message(&self, metadata: &M) -> String {
		T::message(self, metadata)
	}

	fn labels(&self, metadata: &M) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		T::labels(self, metadata)
	}

	fn notes(&self, metadata: &M) -> Vec<String> {
		T::notes(self, metadata)
	}

	fn diagnostic(&self, metadata: &M) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		T::diagnostic(self, metadata)
	}
}

impl<'t, 'v, M: MaybeLocated, T: DiagnoseWithVocabulary<M>> Diagnose<M>
	for Contextual<&'t T, &'v TldrVocabulary>
{
	fn message(&self) -> String {
		self.0.message(self.1)
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		self.0.labels(self.1)
	}

	fn notes(&self) -> Vec<String> {
		self.0.notes(self.1)
	}

	fn diagnostic(&self) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		self.0.diagnostic(self.1)
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
	fn message(&self, vocabulary: &TldrVocabulary) -> String {
		self.value().message(self.metadata(), vocabulary)
	}

	fn labels(
		&self,
		vocabulary: &TldrVocabulary,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		self.value().labels(self.metadata(), vocabulary)
	}

	fn notes(&self, vocabulary: &TldrVocabulary) -> Vec<String> {
		self.value().notes(self.metadata(), vocabulary)
	}

	fn diagnostic(
		&self,
		vocabulary: &TldrVocabulary,
	) -> codespan_reporting::diagnostic::Diagnostic<M::File> {
		self.value().diagnostic(self.metadata(), vocabulary)
	}
}
