use std::cmp::Ordering;

use locspan::Meta;
use treeldr::{metadata::Merge, vocab::Object, Id};

use crate::{
	context::MapIds, functional_property_value, rdf, resource::BindingValueRef, Error,
	FunctionalPropertyValue,
};

pub use treeldr::layout::restriction::Property;

pub mod container;
pub mod primitive;

/// Layout restriction.
#[derive(Debug, Clone, PartialEq)]
pub struct Definition<M> {
	restriction: FunctionalPropertyValue<Restriction, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			restriction: FunctionalPropertyValue::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn restriction(&self) -> &FunctionalPropertyValue<Restriction, M> {
		&self.restriction
	}

	pub fn restriction_mut(&mut self) -> &mut FunctionalPropertyValue<Restriction, M> {
		&mut self.restriction
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			restriction: self.restriction.iter(),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(Id, Id) -> Option<Ordering>,
		prop: Property,
		value: Meta<Object<M>, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::ExclusiveMaximum => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_numeric(value)?.map(|n| {
					Restriction::Primitive(primitive::Restriction::Numeric(
						primitive::Numeric::ExclusiveMaximum(n),
					))
				}),
			),
			Property::ExclusiveMinimum => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_numeric(value)?.map(|n| {
					Restriction::Primitive(primitive::Restriction::Numeric(
						primitive::Numeric::ExclusiveMinimum(n),
					))
				}),
			),
			Property::InclusiveMaximum => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_numeric(value)?.map(|n| {
					Restriction::Primitive(primitive::Restriction::Numeric(
						primitive::Numeric::InclusiveMaximum(n),
					))
				}),
			),
			Property::InclusiveMinimum => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_numeric(value)?.map(|n| {
					Restriction::Primitive(primitive::Restriction::Numeric(
						primitive::Numeric::InclusiveMinimum(n),
					))
				}),
			),
			Property::MaxLength => todo!(),
			Property::MinLength => todo!(),
			Property::Pattern => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_regexp(value)?.map(|p| {
					Restriction::Primitive(primitive::Restriction::String(
						primitive::String::Pattern(p),
					))
				}),
			),
			Property::MaxCardinality => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_non_negative_integer(value)?.map(|n| {
					Restriction::Container(container::ContainerRestriction::Cardinal(
						container::cardinal::Restriction::Max(n),
					))
				}),
			),
			Property::MinCardinality => self.restriction.insert(
				None,
				prop_cmp,
				rdf::from::expect_non_negative_integer(value)?.map(|n| {
					Restriction::Container(container::ContainerRestriction::Cardinal(
						container::cardinal::Restriction::Min(n),
					))
				}),
			),
		}

		Ok(())
	}

	pub fn build(&self) -> Result<Meta<Restriction, M>, Error<M>>
	where
		M: Clone + Merge,
	{
		self.restriction
			.clone()
			.try_unwrap()
			.map_err(|_| {
				todo!() // conflicting restriction
			})?
			.into_required()
			.ok_or_else(|| {
				todo!() // missing restriction
			})
			.map(treeldr::RequiredFunctionalPropertyValue::into_meta_value)
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(treeldr::Id, Option<crate::Property>) -> treeldr::Id) {
		self.restriction.map_ids(f)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Restriction {
	Primitive(primitive::Restriction),
	Container(container::ContainerRestriction),
}

impl Restriction {
	pub fn as_binding(&self) -> ClassBindingRef {
		match self {
			Self::Primitive(r) => ClassBindingRef::Primitive(r.as_binding()),
			Self::Container(r) => ClassBindingRef::Container(r.as_binding_ref()),
		}
	}
}

impl MapIds for Restriction {
	fn map_ids(&mut self, _f: impl Fn(treeldr::Id, Option<crate::Property>) -> treeldr::Id) {
		// nothing.
	}
}

#[derive(Clone)]
pub struct Restrictions<M> {
	pub primitive: primitive::Restrictions<M>,
	pub container: container::ContainerRestrictions<M>,
}

impl<M> Default for Restrictions<M> {
	fn default() -> Self {
		Self {
			primitive: primitive::Restrictions::default(),
			container: container::ContainerRestrictions::default(),
		}
	}
}

impl<M> Restrictions<M> {
	pub fn is_empty(&self) -> bool {
		self.primitive.is_empty() && self.container.is_empty()
	}

	pub fn into_primitive(self) -> primitive::Restrictions<M> {
		self.primitive
	}

	pub fn into_container(self) -> container::ContainerRestrictions<M> {
		self.container
	}

	pub fn insert(&mut self, Meta(restriction, meta): Meta<Restriction, M>) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		match restriction {
			Restriction::Primitive(r) => {
				self.primitive.insert(Meta(r, meta));
				Ok(())
			}
			Restriction::Container(r) => self.container.insert(Meta(r, meta)).map_err(|_| todo!()),
		}
	}

	#[allow(clippy::should_implement_trait)]
	pub fn into_iter(self) -> impl DoubleEndedIterator<Item = Meta<Restriction, M>> {
		self.primitive
			.into_iter()
			.map(|m| m.map(Restriction::Primitive))
			.chain(
				self.container
					.into_iter()
					.map(|m| m.map(Restriction::Container)),
			)
	}
}

pub enum ClassBindingRef<'a> {
	Primitive(primitive::BindingRef<'a>),
	Container(container::BindingRef<'a>),
}

pub type BindingRef<'a> = ClassBindingRef<'a>;

impl<'a> ClassBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::Primitive(b) => b.property(),
			Self::Container(b) => b.property(),
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Primitive(b) => b.value(),
			Self::Container(container::BindingRef::Cardinal(
				treeldr::layout::restriction::cardinal::BindingRef::Min(v),
			)) => BindingValueRef::NonNegativeInteger(v),
			Self::Container(container::BindingRef::Cardinal(
				treeldr::layout::restriction::cardinal::BindingRef::Max(v),
			)) => BindingValueRef::NonNegativeInteger(v),
		}
	}
}

pub struct ClassBindings<'a, M> {
	restriction: functional_property_value::Iter<'a, Restriction, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.restriction
			.next()
			.map(|m| m.value.map(Restriction::as_binding))
	}
}
