use crate::{error, Error};
use locspan::Location;
use treeldr::{Caused, Id, MaybeSet, Name};

pub trait IntersectedWith<F>: Sized {
	fn intersected_with(
		self,
		id: Id,
		other: &Self,
		name: MaybeSet<Name, F>,
		cause: Option<&Location<F>>,
	) -> Result<Self, Error<F>>;
}

impl<F: Clone + Ord> IntersectedWith<F> for treeldr::layout::Literal<F> {
	fn intersected_with(
		self,
		id: Id,
		other: &Self,
		name: MaybeSet<Name, F>,
		cause: Option<&Location<F>>,
	) -> Result<Self, Error<F>> {
		let this = self.into_parts();
		if this.regexp == *other.regexp() {
			Ok(Self::new(
				this.regexp,
				name.unwrap().unwrap_or(this.name),
				this.should_inline && other.should_inline(),
			))
		} else {
			Err(Caused::new(
				error::LayoutIntersectionFailed { id }.into(),
				cause.cloned(),
			))
		}
	}
}
