use super::{Destruct, ListLike, Literal, MapLike, Value, ValueLike};
use crate::{
	rdf::{
		serde::serde_indexed_resource, ResourceBackwardIndex, ResourceForwardIndex, VisitResources,
	},
	TypeRef,
};
use std::hash::Hash;

pub mod map;
pub use map::TypedValueMap;
use serde::{Deserialize, Serialize};
use serde_seeded::{unseeded, DeserializeSeeded, SerializeSeeded};

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct TypedValue<P, R = rdf_types::Term, T = TypeRef<P>> {
	#[seeded(with(unseeded))]
	pub r#type: T,
	pub desc: TypedValueDesc<P, R, T>,
}

impl<P, R, T> TypedValue<P, R, T> {
	pub fn new(r#type: T, desc: impl Into<TypedValueDesc<P, R, T>>) -> Self {
		Self {
			r#type,
			desc: desc.into(),
		}
	}

	pub fn new_resource(r#type: T, resource: R) -> Self {
		Self::new(r#type, TypedValueInnerDesc::Resource(resource))
	}

	pub fn inner_description(&self) -> &TypedValueInnerDesc<P, R, T> {
		self.desc.inner_description()
	}

	pub fn into_inner_description(self) -> TypedValueInnerDesc<P, R, T> {
		self.desc.into_inner_description()
	}

	pub fn as_resource(&self) -> Option<&R> {
		self.desc.as_resource()
	}

	pub fn into_resource(self) -> Option<R> {
		self.desc.into_resource()
	}

	pub fn as_literal(&self) -> Option<&Literal> {
		self.desc.as_literal()
	}
}

impl<P, R: Ord, T> TypedValue<P, R, T> {
	/// Strips the type information and returns a simple tree value.
	pub fn to_untyped(&self) -> Value<R>
	where
		R: Clone,
	{
		self.desc.to_untyped()
	}

	/// Strips the type information and returns a simple tree value.
	pub fn into_untyped(self) -> Value<R> {
		self.desc.into_untyped()
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> TypedValue<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> TypedValue<P, R, U> {
		TypedValue {
			r#type: f(self.r#type),
			desc: self.desc.map_types_ref_mut(f),
		}
	}
}

impl<P, R, T> VisitResources<R> for TypedValue<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.desc.visit_resources_ref_mut(f)
	}
}

impl<P, R, T> ValueLike for TypedValue<P, R, T> {
	type Resource = R;
	type List = Vec<Self>;
	type Map = TypedValueMap<P, R, Self, T>;

	fn destruct(&self) -> Destruct<Self> {
		match self.desc.inner_description() {
			TypedValueInnerDesc::Resource(r) => Destruct::Resource(r),
			TypedValueInnerDesc::Literal(l) => Destruct::Literal(l),
			TypedValueInnerDesc::List(l) => Destruct::List(l),
			TypedValueInnerDesc::Map(m) => Destruct::Map(m),
		}
	}
}

impl<P, R, T> ListLike for Vec<TypedValue<P, R, T>> {
	type Resource = R;
	type Value = TypedValue<P, R, T>;

	fn len(&self) -> usize {
		self.len()
	}

	fn iter(&self) -> impl Iterator<Item = &Self::Value> {
		self.as_slice().iter()
	}
}

impl<P, R, T> MapLike for TypedValueMap<P, R, TypedValue<P, R, T>, T> {
	type Resource = R;
	type Value = TypedValue<P, R, T>;

	fn len(&self) -> usize {
		self.len()
	}

	fn iter(&self) -> impl Iterator<Item = (&Self::Value, &Self::Value)> {
		TypedValueMap::iter(self)
	}
}

/// Typed tree value description.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub enum TypedValueDesc<P, R = rdf_types::Term, T = TypeRef<P>> {
	Variant(String, Box<TypedValue<P, R, T>>),
	Constant(TypedValueInnerDesc<P, R, T>),
}

impl<P, R, T> TypedValueDesc<P, R, T> {
	pub fn inner_description(&self) -> &TypedValueInnerDesc<P, R, T> {
		match self {
			Self::Variant(_, inner) => inner.desc.inner_description(),
			Self::Constant(desc) => desc,
		}
	}

	pub fn into_inner_description(self) -> TypedValueInnerDesc<P, R, T> {
		match self {
			Self::Variant(_, inner) => inner.desc.into_inner_description(),
			Self::Constant(desc) => desc,
		}
	}

	pub fn as_resource(&self) -> Option<&R> {
		self.inner_description().as_resource()
	}

	pub fn into_resource(self) -> Option<R> {
		self.into_inner_description().into_resource()
	}

	pub fn as_literal(&self) -> Option<&Literal> {
		self.inner_description().as_literal()
	}
}

impl<P, R: Ord, T> TypedValueDesc<P, R, T> {
	/// Strips the type information and returns a simple tree value.
	pub fn to_untyped(&self) -> Value<R>
	where
		R: Clone,
	{
		self.inner_description().to_untyped()
	}

	/// Strips the type information and returns a simple tree value.
	pub fn into_untyped(self) -> Value<R> {
		self.into_inner_description().into_untyped()
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> TypedValueDesc<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> TypedValueDesc<P, R, U> {
		match self {
			Self::Variant(name, v) => {
				TypedValueDesc::Variant(name, Box::new(v.map_types_ref_mut(f)))
			}
			Self::Constant(v) => TypedValueDesc::Constant(v.map_types_ref_mut(f)),
		}
	}
}

impl<P, R, T> VisitResources<R> for TypedValueDesc<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		match self {
			Self::Variant(_, e) => e.visit_resources_ref_mut(f),
			Self::Constant(desc) => desc.visit_resources_ref_mut(f),
		}
	}
}

impl<P, R, T> From<TypedValueInnerDesc<P, R, T>> for TypedValueDesc<P, R, T> {
	fn from(value: TypedValueInnerDesc<P, R, T>) -> Self {
		Self::Constant(value)
	}
}

pub type TypedMap<P, R = rdf_types::Term, T = TypeRef<P>> =
	TypedValueMap<P, R, TypedValue<P, R, T>, T>;

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub enum TypedValueInnerDesc<P, R = rdf_types::Term, T = TypeRef<P>> {
	/// RDF resource.
	Resource(#[seeded(with(serde_indexed_resource))] R),

	/// Literal value.
	Literal(Literal),

	/// List.
	List(Vec<TypedValue<P, R, T>>),

	/// Record.
	Map(TypedMap<P, R, T>),
}

impl<P, R, T> TypedValueInnerDesc<P, R, T> {
	pub fn as_resource(&self) -> Option<&R> {
		match self {
			Self::Resource(r) => Some(r),
			_ => None,
		}
	}

	pub fn as_literal(&self) -> Option<&Literal> {
		match self {
			Self::Literal(l) => Some(l),
			_ => None,
		}
	}

	pub fn into_resource(self) -> Option<R> {
		match self {
			Self::Resource(r) => Some(r),
			_ => None,
		}
	}
}

impl<P, R: Ord, T> TypedValueInnerDesc<P, R, T> {
	/// Strips the type information and returns a simple tree value.
	pub fn to_untyped(&self) -> Value<R>
	where
		R: Clone,
	{
		match self {
			Self::Resource(r) => Value::Resource(r.clone()),
			Self::Literal(l) => Value::Literal(l.clone()),
			Self::Map(map) => Value::Map(
				map.iter()
					.map(|(k, v)| (k.to_untyped(), v.to_untyped()))
					.collect(),
			),
			Self::List(items) => Value::List(items.iter().map(TypedValue::to_untyped).collect()),
		}
	}

	/// Strips the type information and returns a simple tree value.
	pub fn into_untyped(self) -> Value<R> {
		match self {
			Self::Resource(r) => Value::Resource(r),
			Self::Literal(l) => Value::Literal(l),
			Self::Map(map) => Value::Map(
				map.into_iter()
					.map(|(k, v)| (k.into_untyped(), v.into_untyped()))
					.collect(),
			),
			Self::List(items) => {
				Value::List(items.into_iter().map(TypedValue::into_untyped).collect())
			}
		}
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> TypedValueInnerDesc<P, R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(
		self,
		f: &mut impl FnMut(T) -> U,
	) -> TypedValueInnerDesc<P, R, U> {
		match self {
			Self::Resource(r) => TypedValueInnerDesc::Resource(r),
			Self::Literal(l) => TypedValueInnerDesc::Literal(l),
			Self::List(l) => {
				TypedValueInnerDesc::List(l.into_iter().map(|v| v.map_types_ref_mut(f)).collect())
			}
			Self::Map(m) => TypedValueInnerDesc::Map(
				m.into_iter()
					.map(|(k, v)| (k.map_types_ref_mut(f), v.map_types_ref_mut(f)))
					.collect(),
			),
		}
	}
}

impl<P, R, T> VisitResources<R> for TypedValueInnerDesc<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		match self {
			Self::Resource(r) => f(r),
			Self::Literal(_) => (),
			Self::List(l) => l.visit_resources_ref_mut(f),
			Self::Map(m) => m.visit_resources_ref_mut(f),
		}
	}
}
