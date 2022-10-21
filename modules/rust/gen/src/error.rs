use crate::Context;
use contextual::WithContext;
use rdf_types::Vocabulary;
use shelves::Ref;
use std::fmt;
use thiserror::Error;
use treeldr::{BlankIdIndex, IriIndex};

#[derive(Error)]
pub enum Error<M> {
	UnreachableType(Ref<treeldr::layout::Definition<M>>),
}

impl<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M> crate::fmt::Display<V, M>
	for Error<M>
{
	fn fmt(&self, context: &Context<V, M>, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::UnreachableType(layout_ref) => {
				let layout = context.model().layouts().get(*layout_ref).unwrap();
				let id = layout.id();

				write!(f, "unbound layout `{}`", id.with(context.vocabulary()))
			}
		}
	}
}
