use crate::{
	metadata::MaybeLocated,
	reporting::{Diagnose, MetadataDiagnostic},
	Vocabulary,
};

pub trait AnyError<M: MetadataDiagnostic> {
	fn message(&self, vocab: &Vocabulary) -> String;

	fn labels(&self, _vocab: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<M::FileId>> {
		Vec::new()
	}

	fn notes(&self, _vocab: &Vocabulary) -> Vec<String> {
		Vec::new()
	}
}

macro_rules! errors {
	{ $( $mod_id:ident :: $id:ident $(<$arg:ident>)? ),* } => {
		$(
			pub mod $mod_id;
			pub use $mod_id::$id;
		)*

		#[derive(Debug)]
		pub enum Error<M> {
			$(
				$id( $id $(<$arg>)? )
			),*
		}

		$(
			impl<M> From<$id $(<$arg>)?> for Error<M> {
				fn from(e: $id $(<$arg>)?) -> Self {
					Self::$id(e)
				}
			}
		)*

		impl<'c, 'a, M: MetadataDiagnostic + MaybeLocated<File = M::FileId>> Diagnose<M> for WithVocabulary<'c, 'a, M> where M::File: Clone {
			fn message(&self) -> String {
				match self.error() {
					$(
						Error::$id(e) => AnyError::<M>::message(e, self.vocabulary())
					),*
				}
			}

			fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<M::FileId>> {
				match self.error() {
					$(
						Error::$id(e) => AnyError::<M>::labels(e, self.vocabulary())
					),*
				}
			}

			fn notes(&self) -> Vec<String> {
				match self.error() {
					$(
						Error::$id(e) => AnyError::<M>::notes(e, self.vocabulary())
					),*
				}
			}
		}
	};
}

errors! {
	node_unknown::NodeUnknown,
	node_invalid_type::NodeInvalidType<M>
}

impl<M> Error<M> {
	pub fn with_vocabulary<'c>(&self, vocab: &'c Vocabulary) -> WithVocabulary<'c, '_, M> {
		WithVocabulary(vocab, self)
	}
}

/// Caused error with contextual information.
pub struct WithVocabulary<'c, 'a, M>(&'c Vocabulary, &'a Error<M>);

impl<'c, 'a, M> WithVocabulary<'c, 'a, M> {
	fn vocabulary(&self) -> &'c Vocabulary {
		self.0
	}

	fn error(&self) -> &'a Error<M> {
		self.1
	}
}
