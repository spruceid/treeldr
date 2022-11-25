use crate::Context;
use contextual::WithContext;
use rdf_types::Vocabulary;
use std::fmt;
use treeldr::{BlankIdIndex, IriIndex, TId};

pub enum Error {
	UnreachableType(TId<treeldr::Layout>),
}

impl<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M> crate::fmt::Display<V, M> for Error {
	fn fmt(&self, context: &Context<V, M>, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UnreachableType(layout_ref) => {
				let layout = context.model().get(*layout_ref).unwrap();
				let id = layout.id();

				write!(f, "unbound layout `{}`", id.with(context.vocabulary()))
			}
		}
	}
}
