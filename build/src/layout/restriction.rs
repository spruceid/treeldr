use locspan::Meta;
use treeldr::metadata::Merge;

use crate::{Error, Single, single};

pub mod primitive;
pub mod container;

/// Layout restriction.
#[derive(Debug, Clone, PartialEq)]
pub struct Definition<M> {
	restriction: Single<Restriction, M>
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self { restriction: Single::default() }
	}

	pub fn build(&self) -> Result<Meta<Restriction, M>, Error<M>> where M: Clone {
		self.restriction.clone().try_unwrap().map_err(|_| {
			todo!() // conflicting restriction
		})?.ok_or_else(|| {
			todo!() // missing restriction
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Restriction {
	Primitive(primitive::Restriction),
	Container(container::Restriction)
}

impl Restriction {
	pub fn as_binding<'a, M>(&'a self, meta: &'a M) -> BindingRef<'a, M> {
		match self {
			Self::Primitive(r) => BindingRef::Primitive(r.as_binding(meta)),
			Self::Container(r) => BindingRef::Container(r.as_binding(meta))
		}
	}
}

#[derive(Clone)]
pub struct Restrictions<M> {
	pub primitive: primitive::Restrictions<M>,
	pub container: container::Restrictions<M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			primitive: primitive::Restrictions::default(),
			container: container::Restrictions::default(),
		}
	}
}

impl<M> Restrictions<M> {
	pub fn into_primitive(self) -> primitive::Restrictions<M> {
		self.primitive
	}

	pub fn into_container(self) -> container::Restrictions<M> {
		self.container
	}

	pub fn insert(&mut self, Meta(restriction, meta): Meta<Restriction, M>) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		match restriction {
			Restriction::Primitive(r) => Ok(self.primitive.insert(Meta(r, meta))),
			Restriction::Container(r) => self.container.insert(Meta(r, meta)).map_err(|_| {
				todo!()
			})
		}
	}
}

pub enum BindingRef<'a, M> {
	Primitive(primitive::BindingRef<'a, M>),
	Container(container::BindingRef<'a, M>)
}

pub struct Bindings<'a, M> {
	restriction: single::Iter<'a, Restriction, M>
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = BindingRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.restriction.next().map(|Meta(r, m)| r.as_binding(m))
	}
}