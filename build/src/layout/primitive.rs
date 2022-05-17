use std::collections::BTreeMap;
use locspan::Location;
pub use treeldr::{
	Causes,
	MaybeSet,
	layout::{
		Primitive,
		primitive::RegExp
	}
};
use crate::Error;

#[derive(Clone, Debug)]
pub struct Restricted<F> {
	primitive: MaybeSet<Primitive, F>,
	restrictions: Restrictions<F>
}

impl<F> Default for Restricted<F> {
	fn default() -> Self {
		Self {
			primitive: MaybeSet::default(),
			restrictions: Restrictions::default()
		}
	}
}

impl<F> PartialEq for Restricted<F> {
	fn eq(&self, other: &Self) -> bool {
		self.primitive.value() == other.primitive.value()
	}
}

impl<F> Eq for Restricted<F> {}

impl<F> Restricted<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn unrestricted(p: Primitive, causes: impl Into<Causes<F>>) -> Self {
		Self {
			primitive: MaybeSet::new(p, causes),
			restrictions: Restrictions::default()
		}
	}

	pub fn primitive(&self) -> &MaybeSet<Primitive, F> {
		&self.primitive
	}

	pub fn set_primitive(&mut self, primitive: Primitive, cause: Option<Location<F>>) -> Result<(), Error<F>> where F: Ord {
		self.primitive.try_set(primitive, cause, |found, expected, found_causes, expected_causes| {
			todo!()
		})
	}

	pub fn restrictions(&self) -> &Restrictions<F> {
		&self.restrictions
	}

	pub fn restrictions_mut(&mut self) -> &mut Restrictions<F> {
		&mut self.restrictions
	}

	pub fn try_unify(self, other: Self) -> Result<Self, Error<F>> {
		todo!()
	}

	pub fn build(self) -> treeldr::layout::BoundedPrimitive {
		todo!()
	}
}

impl<F: Ord> From<Primitive> for Restricted<F> {
	fn from(p: Primitive) -> Self {
		Self {
			primitive: MaybeSet::new(p, None),
			restrictions: Restrictions::default()
		}
	}
}

#[derive(Clone, Debug)]
pub struct Restrictions<F> {
	map: BTreeMap<Restriction, Causes<F>>
}

impl<F> Default for Restrictions<F> {
	fn default() -> Self {
		Self {
			map: BTreeMap::default()
		}
	}
}

impl<F> PartialEq for Restrictions<F> {
	fn eq(&self, other: &Self) -> bool {
		self.map.len() == other.map.len() && self.map.keys().zip(other.map.keys()).all(|(a, b)| a == b)
	}
}

impl<F> Eq for Restrictions<F> {}

impl<F> Restrictions<F> {
	pub fn insert(&mut self, restriction: Restriction, causes: impl Into<Causes<F>>) {
		todo!()
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Restriction {
	Pattern(RegExp)
}

// pub trait IntersectedWith<F>: Sized {
// 	fn intersected_with(
// 		self,
// 		id: Id,
// 		other: &Self,
// 		name: MaybeSet<Name, F>,
// 		cause: Option<&Location<F>>,
// 	) -> Result<Self, Error<F>>;
// }

// impl<F: Clone + Ord> IntersectedWith<F> for treeldr::layout::Literal<F> {
// 	fn intersected_with(
// 		self,
// 		id: Id,
// 		other: &Self,
// 		name: MaybeSet<Name, F>,
// 		cause: Option<&Location<F>>,
// 	) -> Result<Self, Error<F>> {
// 		let this = self.into_parts();
// 		if this.regexp == *other.regexp() {
// 			Ok(Self::new(
// 				this.regexp,
// 				name.unwrap().unwrap_or(this.name),
// 				this.should_inline && other.should_inline(),
// 			))
// 		} else {
// 			Err(Caused::new(
// 				error::LayoutIntersectionFailed { id }.into(),
// 				cause.cloned(),
// 			))
// 		}
// 	}
// }
