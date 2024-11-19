use crate::{display::DisplaySignature, Attributes, Registry, Value};
use core::fmt;
use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug, hash::Hash};

mod resource;
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Type<R, T = TypeRef> {
	pub attributes: Attributes<R>,
	pub desc: TypeDesc<R, T>,
}

impl<R, T> Type<R, T> {
	pub fn new(attributes: Attributes<R>, desc: TypeDesc<R, T>) -> Self {
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

	pub fn as_list(&self) -> Option<&List<T>> {
		self.desc.as_list()
	}

	pub fn as_struct(&self) -> Option<&Map<R, T>> {
		self.desc.as_struct()
	}

	pub fn visit_type_components(&self, f: impl FnMut(&T, bool)) {
		self.desc.visit_type_components(f);
	}
}

impl<R, T> From<TypeDesc<R, T>> for Type<R, T> {
	fn from(desc: TypeDesc<R, T>) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeDesc<R, T = TypeRef> {
	/// RDF resource.
	Resource(Resource<R>),

	/// Literal value.
	Literal(LiteralType),

	/// List.
	List(List<T>),

	/// Map.
	Map(Map<R, T>),

	/// Enum.
	Enum(Enum<R, T>),

	/// Any.
	Any,
}

impl<R, T> TypeDesc<R, T> {
	pub fn resource() -> Self {
		Self::Resource(Resource::default())
	}

	pub fn is_resource(&self) -> bool {
		matches!(self, Self::Resource(_))
	}

	pub fn as_list(&self) -> Option<&List<T>> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}

	pub fn as_struct(&self) -> Option<&Map<R, T>> {
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
					f(&field.type_, false);
				}
			}
			Self::Enum(ty) => {
				for variant in ty.variants.values() {
					f(&variant.type_, false);
				}
			}
		}
	}
}

pub trait TypeUnifier<T, U> {
	type Target;

	fn left_key_type(&mut self, t: &T) -> TypeRef;

	fn right_key_type(&mut self, u: &U) -> TypeRef;

	fn unify(&mut self, t: &T, u: &U) -> Self::Target;
}

impl<R: Clone + Ord, T> Type<R, T> {
	pub fn unify_with<U, V>(
		&self,
		registry: &Registry<R>,
		unifier: &mut impl TypeUnifier<T, U, Target = V>,
		other: &Type<R, U>,
	) -> Result<Type<R, V>, TypeUnificationError>
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

impl<R: Clone + Ord, T> TypeDesc<R, T> {
	pub fn unify_with<U, V>(
		&self,
		registry: &Registry<R>,
		unifier: &mut impl TypeUnifier<T, U, Target = V>,
		other: &TypeDesc<R, U>,
	) -> Result<TypeDesc<R, V>, TypeUnificationError>
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
			(Self::List(a), TypeDesc::List(b)) => Ok(TypeDesc::List(List {
				prefix: if a.prefix.len() == b.prefix.len() {
					a.prefix
						.iter()
						.zip(&b.prefix)
						.map(|(a, b)| unifier.unify(a, b))
						.collect()
				} else {
					return Err(TypeUnificationError);
				},
				rest: match (&a.rest, &b.rest) {
					(Some(a), Some(b)) => Some(unifier.unify(a, b)),
					(None, None) => None,
					_ => return Err(TypeUnificationError),
				},
			})),
			(Self::Map(a), TypeDesc::Map(b)) => {
				if a.fields.len() == b.fields.len() {
					let mut fields = BTreeMap::new();

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
										Field {
											attributes,
											type_: unifier.unify(&a_field.type_, &b_field.type_),
											required: a_field.required,
										},
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
									Variant {
										attributes: a_variant
											.attributes
											.unify_with(&b_variant.attributes)?,
										type_: unifier.unify(&a_variant.type_, &b_variant.type_),
									},
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

impl<R: Ord> Type<R> {
	pub fn subtype_cmp(&self, registry: &Registry<R>, other: &Self) -> Option<Ordering> {
		self.desc.subtype_cmp(registry, &other.desc)
	}

	pub fn contains(&self, registry: &Registry<R>, value: &Value<R>) -> bool {
		self.desc.contains(registry, value)
	}
}

impl<R: Ord> TypeDesc<R> {
	pub fn subtype_cmp(&self, registry: &Registry<R>, other: &Self) -> Option<Ordering> {
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

	pub fn contains(&self, registry: &Registry<R>, value: &Value<R>) -> bool {
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

impl<R: fmt::Display> DisplaySignature<R> for Type<R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		self.desc.fmt_signature(context, f)
	}
}

impl<R: fmt::Display> DisplaySignature<R> for TypeDesc<R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
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
