use crate::{
	display::DisplaySignature, rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources}, value::TypedValueDesc, Attributes, Registry, Value
};
use core::fmt;
use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug, hash::Hash};

mod resource;
use btree_indexmap::BTreeIndexMap;
use map::Field;
use r#enum::Variant;
pub use resource::Resource;

mod literal;
pub use literal::*;

mod list;
pub use list::*;

pub mod map;
pub use map::Map;

pub mod r#enum;
pub use r#enum::Enum;

mod r#ref;
pub use r#ref::*;

mod build_in;
pub use build_in::*;

mod allocator;
pub use allocator::TypeAllocator;
use serde::{Deserialize, Serialize};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>), bounds(T: Ord))
)]
pub struct Type<P, R, T = TypeRef<P>> {
	pub attributes: Attributes<R>,
	pub desc: TypeDesc<P, R, T>,
}

impl<P, R, T> Type<P, R, T> {
	pub fn new(attributes: Attributes<R>, desc: TypeDesc<P, R, T>) -> Self {
		Self { attributes, desc }
	}

	pub fn resource() -> Self {
		Self {
			attributes: Attributes::new(),
			desc: TypeDesc::resource(),
		}
	}

	pub fn is_resource(&self) -> bool {
		self.desc.is_resource()
	}

	pub fn as_list(&self) -> Option<&List<P, T>> {
		self.desc.as_list()
	}

	pub fn as_struct(&self) -> Option<&Map<P, R, T>> {
		self.desc.as_struct()
	}

	pub fn visit_type_components(&self, f: impl FnMut(&T, bool)) {
		self.desc.visit_type_components(f);
	}
}

impl<P, R, T> From<TypeDesc<P, R, T>> for Type<P, R, T> {
	fn from(desc: TypeDesc<P, R, T>) -> Self {
		Self {
			attributes: Attributes::new(),
			desc,
		}
	}
}

impl<R: Clone + PartialEq> Attributes<R> {
	pub fn unify_with(&self, other: &Self) -> Result<Self, TypeUnificationError> {
		if self.is_empty() {
			Ok(other.clone())
		} else if other.is_empty() || self == other {
			Ok(self.clone())
		} else {
			Err(TypeUnificationError)
		}
	}
}

impl<P, R, T> VisitResources<R> for Type<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.attributes.visit_resources_ref_mut(f);
		self.desc.visit_resources_ref_mut(f)
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>), bounds(T: Ord))
)]
pub enum TypeDesc<P, R, T = TypeRef<P>> {
	/// RDF resource.
	Resource(Resource<R>),

	/// Literal value.
	Literal(LiteralType),

	/// List.
	List(List<P, T>),

	/// Map.
	Map(Map<P, R, T>),

	/// Enum.
	Enum(Enum<P, R, T>),

	/// Any.
	Any,
}

impl<P, R, T> TypeDesc<P, R, T> {
	pub fn resource() -> Self {
		Self::Resource(Resource::default())
	}

	pub fn is_resource(&self) -> bool {
		matches!(self, Self::Resource(_))
	}

	pub fn as_list(&self) -> Option<&List<P, T>> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}

	pub fn as_struct(&self) -> Option<&Map<P, R, T>> {
		match self {
			Self::Map(m) => Some(m),
			_ => None,
		}
	}

	pub fn visit_type_components(&self, mut f: impl FnMut(&T, bool)) {
		match self {
			Self::Any | Self::Resource(_) | Self::Literal(_) => (),
			Self::List(ty) => {
				for c_ref in &ty.prefix {
					f(c_ref, false);
				}

				if let Some(c_ref) = &ty.rest {
					f(c_ref, false);
				}
			}
			Self::Map(ty) => {
				for (key, field) in &ty.fields {
					f(key, true);
					f(&field.r#type, false);
				}
			}
			Self::Enum(ty) => {
				for variant in ty.variants.values() {
					f(&variant.r#type, false);
				}
			}
		}
	}
}

impl<P, R, T> VisitResources<R> for TypeDesc<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		match self {
			Self::Resource(r) => r.visit_resources_ref_mut(f),
			Self::Literal(_) => (),
			Self::List(_) => (),
			Self::Map(m) => m.visit_resources_ref_mut(f),
			Self::Enum(e) => e.visit_resources_ref_mut(f),
			Self::Any => (),
		}
	}
}

impl<P, R> TypeDesc<P, R> {
	pub fn to_singleton_value(&self, registry: &impl Registry<PackageId = P, Resource = R>) -> Option<TypedValueDesc<P, R>> where P: Copy, R: Clone + Ord {
		match self {
			Self::Any => None,
			Self::Resource(r) => r.to_singleton_value().map(TypedValueDesc::resource),
			Self::Literal(l) => l.to_singleton_value().map(TypedValueDesc::literal),
			Self::List(l) => l.to_singleton_value(registry).map(TypedValueDesc::list),
			Self::Map(m) => m.to_singleton_value(registry).map(TypedValueDesc::map),
			Self::Enum(_) => None
		}
	}
}

pub trait TypeUnifier<P, T, U> {
	type Target;

	fn left_key_type(&mut self, t: &T) -> TypeRef<P>;

	fn right_key_type(&mut self, u: &U) -> TypeRef<P>;

	fn unify(&mut self, t: &T, u: &U) -> Self::Target;
}

impl<P: Copy + Ord + Hash, R: Clone + Ord, T> Type<P, R, T> {
	pub fn unify_with<U, V>(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		unifier: &mut impl TypeUnifier<P, T, U, Target = V>,
		other: &Type<P, R, U>,
	) -> Result<Type<P, R, V>, TypeUnificationError>
	where
		V: Ord,
	{
		let attributes = if self.attributes.is_empty() {
			other.attributes.clone()
		} else if other.attributes.is_empty() || self.attributes == other.attributes {
			self.attributes.clone()
		} else {
			return Err(TypeUnificationError);
		};

		let desc = self.desc.unify_with(registry, unifier, &other.desc)?;

		Ok(Type::new(attributes, desc))
	}
}

impl<P: Copy + Ord + Hash, R: Clone + Ord, T> TypeDesc<P, R, T> {
	pub fn unify_with<U, V>(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		unifier: &mut impl TypeUnifier<P, T, U, Target = V>,
		other: &TypeDesc<P, R, U>,
	) -> Result<TypeDesc<P, R, V>, TypeUnificationError>
	where
		V: Ord,
	{
		match (self, other) {
			(Self::Any, TypeDesc::Any) => Ok(TypeDesc::Any),
			(Self::Resource(a), TypeDesc::Resource(b)) => {
				if a == b {
					Ok(TypeDesc::Resource(a.clone()))
				} else {
					Err(TypeUnificationError)
				}
			}
			(Self::Literal(a), TypeDesc::Literal(b)) => match a.subtype_cmp(b) {
				Some(Ordering::Equal) => Ok(TypeDesc::Literal(a.clone())),
				_ => Err(TypeUnificationError),
			},
			(Self::List(a), TypeDesc::List(b)) => Ok(TypeDesc::List(List::new(
				if a.prefix.len() == b.prefix.len() {
					a.prefix
						.iter()
						.zip(&b.prefix)
						.map(|(a, b)| unifier.unify(a, b))
						.collect()
				} else {
					return Err(TypeUnificationError);
				},
				match (&a.rest, &b.rest) {
					(Some(a), Some(b)) => Some(unifier.unify(a, b)),
					(None, None) => None,
					_ => return Err(TypeUnificationError),
				},
			))),
			(Self::Map(a), TypeDesc::Map(b)) => {
				if a.fields.len() == b.fields.len() {
					let mut fields = BTreeIndexMap::new();

					for (a_key, a_field) in &a.fields {
						let a_key_type = unifier.left_key_type(a_key);
						let b_entry = b.fields.iter().find(|(k, _)| {
							a_key_type
								.subtype_cmp(registry, unifier.right_key_type(k))
								.is_some_and(Ordering::is_eq)
						});

						match b_entry {
							Some((b_key, b_field)) => {
								let attributes =
									a_field.attributes.unify_with(&b_field.attributes)?;

								if a_field.required == b_field.required {
									let key = unifier.unify(a_key, b_key);
									fields.insert(
										key,
										Field::new(
											attributes,
											unifier.unify(&a_field.r#type, &b_field.r#type),
											a_field.required,
										),
									);
								} else {
									return Err(TypeUnificationError);
								}
							}
							None => return Err(TypeUnificationError),
						}
					}

					Ok(TypeDesc::Map(Map { fields }))
				} else {
					Err(TypeUnificationError)
				}
			}
			(Self::Enum(a), TypeDesc::Enum(b)) => {
				if a.variants.len() == b.variants.len() {
					let mut variants = BTreeMap::new();

					for (key, a_variant) in &a.variants {
						match b.variants.get(key) {
							Some(b_variant) => {
								variants.insert(
									key.clone(),
									Variant::new(
										a_variant.attributes.unify_with(&b_variant.attributes)?,
										unifier.unify(&a_variant.r#type, &b_variant.r#type),
									),
								);
							}
							None => return Err(TypeUnificationError),
						}
					}

					Ok(TypeDesc::Enum(Enum { variants }))
				} else {
					Err(TypeUnificationError)
				}
			}
			_ => Err(TypeUnificationError),
		}
	}
}

#[derive(Debug, thiserror::Error)]
#[error("type unification failed")]
pub struct TypeUnificationError;

impl<P, R> Type<P, R> {
	pub fn to_singleton_value(&self, registry: &impl Registry<PackageId = P, Resource = R>) -> Option<TypedValueDesc<P, R>> where P: Copy, R: Clone + Ord {
		self.desc.to_singleton_value(registry)
	}
}

impl<P: Copy + Ord, R: Ord> Type<P, R> {
	pub fn contains(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		value: &Value<R>,
	) -> bool {
		self.desc.contains(registry, value)
	}
}

impl<P: Copy + Ord + Hash, R: Ord> Type<P, R> {
	pub fn subtype_cmp(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other: &Self,
	) -> Option<Ordering> {
		self.desc.subtype_cmp(registry, &other.desc)
	}
}

impl<P: Copy + Ord, R: Ord> TypeDesc<P, R> {
	pub fn contains(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		value: &Value<R>,
	) -> bool {
		match (self, value) {
			(Self::Any, _) => true,
			(Self::Resource(ty), Value::Resource(r)) => ty.contains(r),
			(Self::Literal(ty), Value::Literal(l)) => ty.contains(l),
			(Self::List(ty), Value::List(list)) => ty.contains(registry, list),
			(Self::Map(ty), Value::Map(map)) => ty.contains(registry, map),
			(Self::Enum(ty), value) => ty.contains(registry, value),
			_ => false,
		}
	}
}

impl<P: Copy + Ord + Hash, R: Ord> TypeDesc<P, R> {
	pub fn subtype_cmp(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other: &Self,
	) -> Option<Ordering> {
		match (self, other) {
			(Self::Any, Self::Any) => Some(Ordering::Equal),
			(Self::Any, _) => Some(Ordering::Greater),
			(_, Self::Any) => Some(Ordering::Less),
			(Self::Resource(a), Self::Resource(b)) => a.subtype_cmp(b),
			(Self::Literal(a), Self::Literal(b)) => a.subtype_cmp(b),
			(Self::List(a), Self::List(b)) => a.subtype_cmp(registry, b),
			(Self::Map(a), Self::Map(b)) => a.subtype_cmp(registry, b),
			(Self::Enum(a), b) => a.subtype_cmp(registry, b),
			(a, Self::Enum(b)) => b.subtype_cmp(registry, a).map(Ordering::reverse),
			_ => None,
		}
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for Type<P, R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		self.desc.fmt_signature(context, f)
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for TypeDesc<P, R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self {
			Self::Any => write!(f, "any"),
			Self::Resource(ty) => ty.fmt_signature(context, f),
			Self::Literal(ty) => ty.fmt_signature(context, f),
			Self::List(ty) => ty.fmt_signature(context, f),
			Self::Map(ty) => ty.fmt_signature(context, f),
			Self::Enum(ty) => ty.fmt_signature(context, f),
		}
	}
}

fn refine_cmp(cmp: Ordering, refine_with: Option<Ordering>) -> Option<Ordering> {
	match (cmp, refine_with?) {
		(Ordering::Equal, cmp) => Some(cmp),
		(Ordering::Greater, Ordering::Greater | Ordering::Equal) => Some(Ordering::Greater),
		(Ordering::Less, Ordering::Less | Ordering::Equal) => Some(Ordering::Less),
		_ => None,
	}
}
