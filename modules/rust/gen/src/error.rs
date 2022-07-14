use crate::Context;
use shelves::Ref;
use std::fmt;
use thiserror::Error;

#[derive(Error)]
pub enum Error<F> {
	UnreachableType(Ref<treeldr::layout::Definition<F>>),
}

impl<F> crate::fmt::Display<F> for Error<F> {
	fn fmt(&self, context: &Context<F>, f: &mut fmt::Formatter) -> fmt::Result {
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
