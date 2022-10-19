use crate::Context;
use shelves::Ref;
use std::fmt;
use thiserror::Error;

#[derive(Error)]
pub enum Error<M> {
	UnreachableType(Ref<treeldr::layout::Definition<M>>),
}

impl<M> crate::fmt::Display<M> for Error<M> {
	fn fmt(&self, context: &Context<M>, f: &mut fmt::Formatter) -> fmt::Result {
		use treeldr::vocab::Display;
		match self {
			Self::UnreachableType(layout_ref) => {
				let layout = context.model().layouts().get(*layout_ref).unwrap();
				let id = layout.id();

				write!(f, "unbound layout `{}`", id.display(context.vocabulary()))
			}
		}
	}
}
