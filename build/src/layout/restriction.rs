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
	pub fn as_binding(&self) -> ClassBindingRef {
		match self {
			Self::Primitive(r) => ClassBindingRef::Primitive(r.as_binding()),
			Self::Container(r) => ClassBindingRef::Container(r.as_binding())
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

pub enum ClassBindingRef<'a> {
	Primitive(primitive::BindingRef<'a>),
	Container(container::Binding)
}

pub type BindingRef<'a> = ClassBindingRef<'a>;

pub struct ClassBindings<'a, M> {
	restriction: single::Iter<'a, Restriction, M>
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.restriction.next().map(|m| m.map(Restriction::as_binding))
	}
}