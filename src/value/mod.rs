use core::fmt;
use std::{collections::BTreeMap, hash::Hash, str::FromStr};

mod like;
pub use like::*;

mod literal;
pub use literal::*;

mod typed;
use ll1::Parse;
use serde_seeded::{DeserializeSeeded, SerializeSeeded};
pub use typed::*;

pub mod serde;

use crate::{
	rdf::{
		serde::serde_indexed_resource, ResourceBackwardIndex, ResourceForwardIndex, VisitResources,
	},
	ty::TypeDesc,
	utils::try_map_list,
	Registry, Type, TypeRef,
};

pub type List<R = rdf_types::Term> = Vec<Value<R>>;

pub type Map<R = rdf_types::Term> = BTreeMap<Value<R>, Value<R>>;

/// Untyped tree value.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(R: Eq + Hash)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(R: Clone + Ord))
)]
pub enum Value<R = rdf_types::Term> {
	Resource(#[seeded(with(serde_indexed_resource))] R),
	Literal(Literal),
	Map(Map<R>),
	List(List<R>),
}

impl<R> Default for Value<R> {
	fn default() -> Self {
		Self::unit()
	}
}

impl<R> Value<R> {
	/// Returns the unit value.
	pub fn unit() -> Self {
		Self::Literal(Literal::Unit)
	}

	/// Checks if the value is unit.
	pub fn is_unit(&self) -> bool {
		matches!(self, Self::Literal(Literal::Unit))
	}

	pub fn boolean(value: bool) -> Self {
		Self::Literal(Literal::Boolean(value))
	}

	pub fn number(value: Number) -> Self {
		Self::Literal(Literal::Number(value))
	}

	/// Creates a text string value.
	pub fn string(value: String) -> Self {
		Self::Literal(Literal::TextString(value))
	}

	pub fn is_resource(&self) -> bool {
		matches!(self, Self::Resource(_))
	}

	pub fn as_resource(&self) -> Option<&R> {
		match self {
			Self::Resource(r) => Some(r),
			_ => None,
		}
	}

	pub fn into_resource(self) -> Option<R> {
		match self {
			Self::Resource(r) => Some(r),
			_ => None,
		}
	}

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Self::Literal(l) => l.as_bool(),
			_ => None,
		}
	}

	pub fn as_number(&self) -> Option<&Number> {
		match self {
			Self::Literal(l) => l.as_number(),
			_ => None,
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::Literal(l) => l.as_str(),
			_ => None,
		}
	}

	pub fn into_typed<P>(
		self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		ty_ref: TypeRef<P>,
	) -> Result<TypedValue<P, R>, Value<R>>
	where
		P: Copy,
		R: Clone + Ord + Hash,
	{
		let ty: &Type<P, R> = registry.get_type(ty_ref).unwrap();
		match (&ty.desc, self) {
			(TypeDesc::Any, value) => Ok(TypedValue {
				r#type: ty_ref,
				desc: match value {
					Value::Resource(r) => TypedValueInnerDesc::Resource(r),
					Value::Literal(l) => TypedValueInnerDesc::Literal(l),
					Value::List(l) => try_map_list(l, TypedValue::into_untyped, |_, item| {
						item.into_typed(registry, ty_ref)
					})
					.map(TypedValueInnerDesc::List)
					.map_err(Self::List)?,
					Value::Map(m) => try_map_into_typed(m, TypedValue::into_untyped, |item| {
						item.into_typed(registry, ty_ref)
					})
					.map(TypedValueInnerDesc::Map)
					.map_err(Self::Map)?,
				}
				.into(),
			}),
			(TypeDesc::Resource(_), Self::Resource(r)) => Ok(TypedValue {
				r#type: ty_ref,
				desc: TypedValueInnerDesc::Resource(r).into(),
			}),
			(TypeDesc::Literal(ty), Self::Literal(l)) => {
				if ty.contains(&l) {
					Ok(TypedValue {
						r#type: ty_ref,
						desc: TypedValueInnerDesc::Literal(l).into(),
					})
				} else {
					Err(Self::Literal(l))
				}
			}
			(TypeDesc::List(ty), Self::List(items)) => {
				if items.len() < ty.min_len() {
					return Err(Self::List(items));
				}

				if ty.max_len().is_some_and(|m| items.len() > m) {
					return Err(Self::List(items));
				}

				Ok(TypedValue {
					r#type: ty_ref,
					desc: TypedValueInnerDesc::List(
						items
							.into_iter()
							.enumerate()
							.map(|(i, item)| item.into_typed(registry, ty.item_type(i).unwrap()))
							.collect::<Result<_, _>>()?,
					)
					.into(),
				})
			}
			(TypeDesc::Map(ty), Self::Map(mut map)) => {
				let mut result = TypedValueMap::new();

				for (&key_pattern, field) in &ty.fields {
					let mut found = false;
					let mut entries = std::mem::take(&mut map).into_iter();

					while let Some((key, value)) = entries.next() {
						match key.into_typed(registry, key_pattern) {
							Ok(typed_key) => match value.into_typed(registry, field.r#type) {
								Ok(typed_value) => {
									found = true;
									result.insert(typed_key, typed_value);
								}
								Err(value) => {
									let mut reverted = BTreeMap::new();
									reverted.extend(
										result
											.into_iter()
											.map(|(k, v)| (k.into_untyped(), v.into_untyped())),
									);
									reverted.insert(typed_key.into_untyped(), value);
									reverted.extend(entries);
									return Err(Self::Map(reverted));
								}
							},
							Err(key) => {
								map.insert(key, value);
							}
						}
					}

					if field.required && !found {
						let mut reverted = BTreeMap::new();
						reverted.extend(
							result
								.into_iter()
								.map(|(k, v)| (k.into_untyped(), v.into_untyped())),
						);
						reverted.extend(entries);
						return Err(Self::Map(reverted));
					}
				}

				Ok(TypedValue {
					r#type: ty_ref,
					desc: TypedValueInnerDesc::Map(result).into(),
				})
			}
			(TypeDesc::Enum(e), mut value) => {
				for (name, variant) in &e.variants {
					match std::mem::take(&mut value).into_typed(registry, variant.r#type) {
						Ok(typed_value) => {
							return Ok(TypedValue {
								r#type: ty_ref,
								desc: TypedValueDesc::Variant(name.clone(), Box::new(typed_value)),
							})
						}
						Err(v) => value = v,
					}
				}

				Err(value)
			}
			(_, value) => Err(value),
		}
	}
}

impl<R> VisitResources<R> for Value<R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		match self {
			Self::Resource(r) => f(r),
			Self::Literal(_) => (),
			Self::Map(m) => m.visit_resources_ref_mut(f),
			Self::List(l) => l.visit_resources_ref_mut(f),
		}
	}
}

impl FromStr for Value {
	type Err = ValueParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let ast = crate::syntax::ast::ConstValue::parse_str(s)
			.map_err(|_| ValueParseError(s.to_owned()))?;

		ast.build_unscoped::<()>()
			.map_err(|_| ValueParseError(s.to_owned()))
	}
}

#[derive(Debug, thiserror::Error)]
#[error("invalid value")]
pub struct ValueParseError(pub String);

impl<R> ValueLike for Value<R> {
	type Resource = R;
	type List = Vec<Self>;
	type Map = BTreeMap<Self, Self>;

	fn destruct(&self) -> Destruct<Self> {
		match self {
			Self::Resource(r) => Destruct::Resource(r),
			Self::Literal(l) => Destruct::Literal(l),
			Self::List(l) => Destruct::List(l),
			Self::Map(m) => Destruct::Map(m),
		}
	}
}

impl<R> ListLike for Vec<Value<R>> {
	type Resource = R;
	type Value = Value<R>;

	fn len(&self) -> usize {
		self.len()
	}

	fn iter(&self) -> impl Iterator<Item = &Self::Value> {
		self.as_slice().iter()
	}
}

impl<R> MapLike for BTreeMap<Value<R>, Value<R>> {
	type Resource = R;
	type Value = Value<R>;

	fn len(&self) -> usize {
		self.len()
	}

	fn iter(&self) -> impl Iterator<Item = (&Self::Value, &Self::Value)> {
		self.iter()
	}
}

#[derive(Debug, thiserror::Error)]
#[error("type mismatch")]
pub struct TypeMismatch;

impl<R: fmt::Display> fmt::Display for Value<R> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Resource(r) => r.fmt(f),
			Self::Literal(l) => l.fmt(f),
			Self::List(l) => {
				f.write_str("[")?;
				for (i, v) in l.iter().enumerate() {
					if i > 0 {
						f.write_str(",")?;
					}

					v.fmt(f)?;
				}
				f.write_str("]")
			}
			Self::Map(m) => {
				f.write_str("{")?;
				for (i, (k, v)) in m.iter().enumerate() {
					if i > 0 {
						f.write_str(",")?;
					}

					k.fmt(f)?;
					f.write_str(":")?;
					v.fmt(f)?;
				}
				f.write_str("}")
			}
		}
	}
}

fn try_map_into_typed<P, R: Ord>(
	l: Map<R>,
	revert: impl Fn(TypedValue<P, R>) -> Value<R>,
	f: impl Fn(Value<R>) -> Result<TypedValue<P, R>, Value<R>>,
) -> Result<TypedMap<P, R>, Map<R>> {
	let mut result = TypedMap::new();

	let mut entries = l.into_iter();

	while let Some((ak, av)) = entries.next() {
		match f(ak) {
			Ok(bk) => match f(av) {
				Ok(bv) => {
					result.insert(bk, bv);
				}
				Err(av) => {
					let mut reverted = BTreeMap::new();
					reverted.extend(result.into_iter().map(|(bk, bv)| (revert(bk), revert(bv))));
					reverted.insert(revert(bk), av);
					reverted.extend(entries);
					return Err(reverted);
				}
			},
			Err(ak) => {
				let mut reverted = BTreeMap::new();
				reverted.extend(result.into_iter().map(|(bk, bv)| (revert(bk), revert(bv))));
				reverted.insert(ak, av);
				reverted.extend(entries);
				return Err(reverted);
			}
		}
	}

	Ok(result)
}

#[cfg(test)]
mod tests {
	use super::Number;
	use num_rational::Ratio;

	#[test]
	fn decimal_representation() {
		let vectors = [
			((1, 1), Some("1")),
			((1, 2), Some("0.5")),
			((1, 3), None),
			((1, 4), Some("0.25")),
			((1, 5), Some("0.2")),
			((1, 6), None),
			((1, 7), None),
			((1, 8), Some("0.125")),
			((1, 9), None),
		];

		for ((p, q), expected) in vectors {
			let number = Number::new(Ratio::new(p.into(), q.into()));
			assert_eq!(number.decimal_representation().as_deref(), expected)
		}
	}
}
