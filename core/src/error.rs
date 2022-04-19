use crate::{reporting::Diagnose, Vocabulary};

pub trait AnyError<F> {
	fn message(&self, vocab: &Vocabulary) -> String;

	fn labels(&self, _vocab: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
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
		pub enum Error<F> {
			$(
				$id( $id $(<$arg>)? )
			),*
		}

		$(
			impl<F> From<$id $(<$arg>)?> for Error<F> {
				fn from(e: $id $(<$arg>)?) -> Self {
					Self::$id(e)
				}
			}
		)*

		impl<'c, 'a, F: Clone> Diagnose<F> for WithVocabulary<'c, 'a, F> {
			fn message(&self) -> String {
				match self.error() {
					$(
						Error::$id(e) => AnyError::<F>::message(e, self.vocabulary())
					),*
				}
			}

			fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
				match self.error() {
					$(
						Error::$id(e) => e.labels(self.vocabulary())
					),*
				}
			}

			fn notes(&self) -> Vec<String> {
				match self.error() {
					$(
						Error::$id(e) => AnyError::<F>::notes(e, self.vocabulary())
					),*
				}
			}
		}
	};
}

errors! {
	node_unknown::NodeUnknown,
	node_invalid_type::NodeInvalidType<F>
}

impl<F> Error<F> {
	pub fn with_vocabulary<'c>(&self, vocab: &'c Vocabulary) -> WithVocabulary<'c, '_, F> {
		WithVocabulary(vocab, self)
	}
}

/// Caused error with contextual information.
pub struct WithVocabulary<'c, 'a, F>(&'c Vocabulary, &'a Error<F>);

impl<'c, 'a, F> WithVocabulary<'c, 'a, F> {
	fn vocabulary(&self) -> &'c Vocabulary {
		self.0
	}

	fn error(&self) -> &'a Error<F> {
		self.1
	}
}