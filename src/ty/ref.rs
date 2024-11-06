use core::fmt;
use std::{cmp::Ordering, hash::Hash, ops::Deref};

use crate::{Registry, Value};
use dashmap::DashMap;

use super::Type;

/// Type reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeRef {
	pub namespace: u32,
	pub index: u32
}

impl TypeRef {
	pub fn new(namespace: u32, index: u32) -> Self {
		Self {
			namespace,
			index
		}
	}

	pub fn contains<R: Ord>(&self, registry: &Registry<R>, value: &Value<R>) -> bool {
		registry
			.get_type(*self)
			.is_some_and(|ty| ty.value.contains(registry, value))
	}

	pub fn subtype_cmp<R: Ord>(
		&self,
		registry: &Registry<R>,
		other_ref: TypeRef,
	) -> Option<Ordering> {
		registry.get_type(*self)?.subtype_cmp(registry, other_ref)
	}

	pub fn is_subtype_of<R: Ord>(&self, registry: &Registry<R>, other_ref: TypeRef) -> bool {
		registry
			.get_type(*self)
			.is_some_and(|ty| ty.is_subtype_of(registry, other_ref))
	}

	pub fn display<'a, R>(&self, registry: &'a Registry<R>) -> DisplayedType<'a, R> {
		DisplayedType {
			registry,
			self_: *self
		}
	}
}

impl<R: fmt::Display> DisplayType<R> for TypeRef {
	fn fmt_type(&self, d: &DisplayedType<R>, depth: usize, f: &mut fmt::Formatter) -> fmt::Result {
		if depth > 0 && d.self_ == *self {
			write!(f, "self")
		} else {
			d
				.registry
				.get_type(*self)
				.unwrap()
				.fmt_type(d, depth, f)
		}
	}
}

/// Type definition.
pub struct TypeDefinition<R> {
	value: Type<R>,
	subtype_cmp: DashMap<TypeRef, Option<Ordering>>,
}

impl<R> TypeDefinition<R> {
	pub fn new(value: Type<R>) -> Self {
		Self {
			value,
			subtype_cmp: DashMap::default(),
		}
	}
}

impl<R: Ord> TypeDefinition<R> {
	pub fn subtype_cmp(&self, registry: &Registry<R>, other_ref: TypeRef) -> Option<Ordering> {
		match self.subtype_cmp.get(&other_ref) {
			Some(cmp) => *cmp,
			None => {
				let other = registry.get_type(other_ref)?;
				self.subtype_cmp.insert(other_ref, Some(Ordering::Equal));
				let cmp = self.value.subtype_cmp(registry, other);
				self.subtype_cmp.insert(other_ref, cmp);
				cmp
			}
		}
	}

	pub fn is_subtype_of(&self, registry: &Registry<R>, other_ref: TypeRef) -> bool {
		self.subtype_cmp(registry, other_ref)
			.is_some_and(Ordering::is_le)
	}
}

impl<R> Deref for TypeDefinition<R> {
	type Target = Type<R>;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<R: fmt::Debug> fmt::Debug for TypeDefinition<R> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value.fmt(f)
	}
}

impl<R: fmt::Display> DisplayType<R> for TypeDefinition<R> {
	fn fmt_type(&self, d: &DisplayedType<R>, depth: usize, f: &mut fmt::Formatter) -> fmt::Result {
		self.value.fmt_type(d, depth, f)
	}
}

pub trait DisplayType<R> {
	fn fmt_type(&self, d: &DisplayedType<R>, depth: usize, f: &mut fmt::Formatter) -> fmt::Result;
}

pub struct DisplayedType<'a, R> {
	pub registry: &'a Registry<R>,
	pub self_: TypeRef
}

impl<'a, R: fmt::Display> fmt::Display for DisplayedType<'a, R> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.self_.fmt_type(self, 0, f)
	}
}