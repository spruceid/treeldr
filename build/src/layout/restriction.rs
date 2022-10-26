use locspan::MapLocErr;
use treeldr::metadata::Merge;

use crate::{error, Error};

#[derive(Clone)]
pub struct Restrictions<M> {
	pub primitive: super::primitive::Restrictions<M>,
	pub container: treeldr::layout::container::Restrictions<M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			primitive: super::primitive::Restrictions::default(),
			container: treeldr::layout::container::Restrictions::default(),
		}
	}
}

impl<M> Restrictions<M> {
	pub fn into_primitive(self) -> super::primitive::Restrictions<M> {
		self.primitive
	}

	pub fn into_container(self) -> treeldr::layout::container::Restrictions<M> {
		self.container
	}

	pub fn intersected_with(mut self, other: Self) -> Result<Self, Error<M>>
	where
		M: Clone + Merge,
	{
		self.primitive.unify(other.primitive);
		self.container
			.unify(other.container)
			.map_loc_err(error::Description::LayoutContainerRestrictionConflict)?;
		Ok(self)
	}
}
