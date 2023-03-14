use crate::Context;
use contextual::{DisplayWithContext, WithContext};
use rdf_types::Vocabulary;
use std::fmt;
use treeldr::{BlankIdIndex, IriIndex, TId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("unreachable type")]
	UnreachableType(TId<treeldr::Layout>),

	#[error("unreachable trait")]
	UnreachableTrait(TId<treeldr::Type>),

	#[error("missing required `Default` implementation")]
	MissingDefaultImpl,

	#[error("blank property")]
	BlankProperty(TId<treeldr::Property>),
}

impl<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>> DisplayWithContext<V> for Error {
	fn fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UnreachableType(layout_ref) => {
				write!(f, "unbound layout `{}`", layout_ref.id().with(vocabulary))
			}
			Self::UnreachableTrait(type_ref) => {
				write!(f, "unbound type `{}`", type_ref.id().with(vocabulary))
			}
			Self::MissingDefaultImpl => {
				write!(f, "missing `Default` implementation")
			}
			Self::BlankProperty(prop_ref) => {
				write!(f, "blank property `{}`", prop_ref.id().with(vocabulary))
			}
		}
	}
}

impl<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M> crate::fmt::Display<V, M> for Error {
	fn fmt(&self, context: &Context<V, M>, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		self.with(context.vocabulary()).fmt(f)
	}
}
