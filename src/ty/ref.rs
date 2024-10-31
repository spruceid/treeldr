use core::fmt;
use std::{cmp::Ordering, hash::Hash, ops::Deref};

use crate::{Registry, Value};
use dashmap::DashMap;

use super::Type;

/// Type reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeRef(usize);

impl TypeRef {
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
}

impl From<TypeRef> for usize {
	fn from(value: TypeRef) -> Self {
		value.0
	}
}

impl From<usize> for TypeRef {
	fn from(value: usize) -> Self {
		Self(value)
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
