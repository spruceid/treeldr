use core::fmt;
use std::{cmp::Ordering, hash::Hash, ops::Deref, sync::OnceLock};

use crate::{
	display::DisplaySignature, rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources}, value::TypedValue, Registry, Value
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use super::Type;

/// Type reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeRef<P> {
	pub package: P,
	pub index: u32,
}

impl<P> TypeRef<P> {
	pub fn new(package: P, index: u32) -> Self {
		Self { package, index }
	}

	pub fn to_singleton_value<R>(&self, registry: &impl Registry<PackageId = P, Resource = R>) -> Option<TypedValue<P, R>> where P: Copy, R: Clone + Ord {
		Some(TypedValue::new(
			*self,
			registry.get_type(*self)?.to_singleton_value(registry)?
		))
	}
}

impl<P: Copy + Ord> TypeRef<P> {
	pub fn contains<R: Ord>(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		value: &Value<R>,
	) -> bool {
		registry
			.get_type(*self)
			.is_some_and(|ty| ty.value.contains(registry, value))
	}
}

impl<P: Copy + Ord + Hash> TypeRef<P> {
	pub fn subtype_cmp<R: Ord>(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other_ref: TypeRef<P>,
	) -> Option<Ordering> {
		registry.get_type(*self)?.subtype_cmp(registry, other_ref)
	}

	pub fn is_subtype_of<R: Ord>(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other_ref: TypeRef<P>,
	) -> bool {
		registry
			.get_type(*self)
			.is_some_and(|ty| ty.is_subtype_of(registry, other_ref))
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for TypeRef<P> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		if context.self_ == Some(*self) {
			write!(f, "self")
		} else {
			context
				.registry
				.get_type(*self)
				.unwrap()
				.fmt_signature(&context.inner_self(*self), f)
		}
	}
}

/// Type definition.
#[derive(SerializeSeeded, DeserializeSeeded)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord), bounds(P: Ord)),
	transparent
)]
pub struct TypeDefinition<P, R> {
	value: Type<P, R>,

	#[seeded(skip)]
	subtype_cmp: OnceLock<DashMap<TypeRef<P>, Option<Ordering>>>,
}

impl<P, R> TypeDefinition<P, R> {
	pub fn new(value: Type<P, R>) -> Self {
		Self {
			value,
			subtype_cmp: OnceLock::new(),
		}
	}
}

impl<P: Copy + Ord + Hash, R: Ord> TypeDefinition<P, R> {
	pub fn subtype_cmp(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other_ref: TypeRef<P>,
	) -> Option<Ordering> {
		let map = self.subtype_cmp.get_or_init(Default::default);
		match map.get(&other_ref) {
			Some(cmp) => *cmp,
			None => {
				let other = registry.get_type(other_ref)?;
				map.insert(other_ref, Some(Ordering::Equal));
				let cmp = self.value.subtype_cmp(registry, other);
				map.insert(other_ref, cmp);
				cmp
			}
		}
	}

	pub fn is_subtype_of(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other_ref: TypeRef<P>,
	) -> bool {
		self.subtype_cmp(registry, other_ref)
			.is_some_and(Ordering::is_le)
	}
}

impl<P, R> VisitResources<R> for TypeDefinition<P, R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.value.visit_resources_ref_mut(f)
	}
}

impl<P, R> From<Type<P, R>> for TypeDefinition<P, R> {
	fn from(value: Type<P, R>) -> Self {
		Self::new(value)
	}
}

impl<P, R> Deref for TypeDefinition<P, R> {
	type Target = Type<P, R>;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<P: fmt::Debug, R: fmt::Debug> fmt::Debug for TypeDefinition<P, R> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value.fmt(f)
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for TypeDefinition<P, R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		self.value.fmt_signature(context, f)
	}
}
