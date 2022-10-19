use crate::{reporting::Diagnose, BlankIdIndex, IriIndex};
use locspan::{MaybeLocated, Span};
use rdf_types::Vocabulary;

pub trait AnyError<M: MaybeLocated<Span = Span>> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String;

	fn labels(
		&self,
		_vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		Vec::new()
	}

	fn notes(
		&self,
		_vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<String> {
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

		impl<'c, 'a, M: MaybeLocated<Span=Span>, V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>> Diagnose<M> for WithVocabulary<'c, 'a, M, V> where M::File: Clone {
			fn message(&self) -> String {
				match self.error() {
					$(
						Error::$id(e) => AnyError::<M>::message(e, self.vocabulary())
					),*
				}
			}

			fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
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
	pub fn with_vocabulary<'c, V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		vocab: &'c V,
	) -> WithVocabulary<'c, '_, M, V> {
		WithVocabulary(vocab, self)
	}
}

/// Caused error with contextual information.
pub struct WithVocabulary<'c, 'a, M, V>(&'c V, &'a Error<M>);

impl<'c, 'a, M, V> WithVocabulary<'c, 'a, M, V> {
	fn vocabulary(&self) -> &'c V {
		self.0
	}

	fn error(&self) -> &'a Error<M> {
		self.1
	}
}
