use super::{Destruct, ListLike, Literal, MapLike, Value, ValueLike};
use crate::TypeRef;

pub mod map;
pub use map::TypedMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypedValue<R = rdf_types::Term, T = TypeRef> {
	pub type_: T,
	pub desc: TypedValueDesc<R, T>,
}

impl<R, T> TypedValue<R, T> {
	pub fn new(type_: T, desc: impl Into<TypedValueDesc<R, T>>) -> Self {
		Self {
			type_,
			desc: desc.into(),
		}
	}

	pub fn new_resource(type_: T, resource: R) -> Self {
		Self::new(type_, TypedValueInnerDesc::Resource(resource))
	}

	pub fn inner_description(&self) -> &TypedValueInnerDesc<R, T> {
		self.desc.inner_description()
	}

	pub fn into_inner_description(self) -> TypedValueInnerDesc<R, T> {
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

impl<R: Ord, T> TypedValue<R, T> {
	/// Strips the type information and returns a simple tree value.
	pub fn to_untyped(&self) -> Value<R>
	where
		R: Clone
	{
		self.desc.to_untyped()
	}

	/// Strips the type information and returns a simple tree value.
	pub fn into_untyped(self) -> Value<R> {
		self.desc.into_untyped()
	}

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> TypedValue<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> TypedValue<R, U> {
		TypedValue {
			type_: f(self.type_),
			desc: self.desc.map_types_ref_mut(f)
		}
	}
}

impl<R, T> ValueLike for TypedValue<R, T> {
	type Resource = R;
	type List = Vec<Self>;
	type Map = TypedMap<R, Self, T>;

	fn destruct(&self) -> Destruct<Self> {
		match self.desc.inner_description() {
			TypedValueInnerDesc::Resource(r) => Destruct::Resource(r),
			TypedValueInnerDesc::Literal(l) => Destruct::Literal(l),
			TypedValueInnerDesc::List(l) => Destruct::List(l),
			TypedValueInnerDesc::Map(m) => Destruct::Map(m),
		}
	}
}

impl<R, T> ListLike for Vec<TypedValue<R, T>> {
	type Resource = R;
	type Value = TypedValue<R, T>;

	fn len(&self) -> usize {
		self.len()
	}

	fn iter(&self) -> impl Iterator<Item = &Self::Value> {
		self.as_slice().iter()
	}
}

impl<R, T> MapLike for TypedMap<R, TypedValue<R, T>, T> {
	type Resource = R;
	type Value = TypedValue<R, T>;

	fn len(&self) -> usize {
		self.len()
	}

	fn iter(&self) -> impl Iterator<Item = (&Self::Value, &Self::Value)> {
		TypedMap::iter(self)
	}
}

/// Typed tree value description.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedValueDesc<R = rdf_types::Term, T = TypeRef> {
	Variant(String, Box<TypedValue<R, T>>),
	Constant(TypedValueInnerDesc<R, T>),
}

impl<R, T> TypedValueDesc<R, T> {
	pub fn inner_description(&self) -> &TypedValueInnerDesc<R, T> {
		match self {
			Self::Variant(_, inner) => inner.desc.inner_description(),
			Self::Constant(desc) => desc,
		}
	}

	pub fn into_inner_description(self) -> TypedValueInnerDesc<R, T> {
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

impl<R: Ord, T> TypedValueDesc<R, T> {
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

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> TypedValueDesc<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> TypedValueDesc<R, U> {
		match self {
			Self::Variant(name, v) => TypedValueDesc::Variant(name, Box::new(v.map_types_ref_mut(f))),
			Self::Constant(v) => TypedValueDesc::Constant(v.map_types_ref_mut(f))
		}
	}
}

impl<R, T> From<TypedValueInnerDesc<R, T>> for TypedValueDesc<R, T> {
	fn from(value: TypedValueInnerDesc<R, T>) -> Self {
		Self::Constant(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedValueInnerDesc<R = rdf_types::Term, T = TypeRef> {
	/// RDF resource.
	Resource(R),

	/// Literal value.
	Literal(Literal),

	/// List.
	List(Vec<TypedValue<R, T>>),

	/// Record.
	Map(TypedMap<R, TypedValue<R, T>, T>),
}

impl<R, T> TypedValueInnerDesc<R, T> {
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

impl<R: Ord, T> TypedValueInnerDesc<R, T> {
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

	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> TypedValueInnerDesc<R, U> {
		self.map_types_ref_mut(&mut f)
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> TypedValueInnerDesc<R, U> {
		match self {
			Self::Resource(r) => TypedValueInnerDesc::Resource(r),
			Self::Literal(l) => TypedValueInnerDesc::Literal(l),
			Self::List(l) => TypedValueInnerDesc::List(l.into_iter().map(|v| v.map_types_ref_mut(f)).collect()),
			Self::Map(m) => TypedValueInnerDesc::Map(m.into_iter().map(|(k, v)| {
				(
					k.map_types_ref_mut(f),
					v.map_types_ref_mut(f)
				)
			}).collect())
		}
	}
}
