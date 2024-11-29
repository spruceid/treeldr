use serde::{ser::SerializeSeq, Deserialize};
use std::{collections::BTreeMap, marker::PhantomData};

use super::{index::ResourceForwardIndex, RdfContext, RdfContextMut, ResourceBackwardIndex};

pub struct RdfSeed<'a, C: ?Sized, T> {
	rdf: &'a mut C,
	t: PhantomData<T>,
}

impl<'a, C: ?Sized, T> RdfSeed<'a, C, T> {
	pub fn new(rdf: &'a mut C) -> Self {
		Self {
			rdf,
			t: PhantomData,
		}
	}
}

impl<'a, 'de, C, T> serde::de::DeserializeSeed<'de> for RdfSeed<'a, C, T>
where
	C: ?Sized + RdfContextMut,
	T: DeserializeRdf<'de, C::Resource>,
{
	type Value = T;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		T::deserialize_rdf(self.rdf, deserializer)
	}
}

pub trait DeserializeRdf<'de, R>: Sized {
	fn deserialize_rdf<D>(
		rdf: &mut (impl ?Sized + RdfContextMut<Resource = R>),
		deserializer: D,
	) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>;
}

pub trait SerializeRdf<R> {
	fn serialize_rdf<S>(
		&self,
		rdf: &impl RdfContext<Resource = R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer;
}

pub struct WithRdf<'a, T, C> {
	pub value: &'a T,
	pub rdf: &'a C,
}

impl<'a, T, C> serde::Serialize for WithRdf<'a, T, C>
where
	C: RdfContext,
	T: SerializeRdf<C::Resource>,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.value.serialize_rdf(self.rdf, serializer)
	}
}

pub trait DeserializeIndexed<'de, R>: Sized {
	fn deserialize_indexed<D>(
		index: &ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>;
}

impl<'de, R, T> DeserializeIndexed<'de, R> for Vec<T>
where
	T: DeserializeIndexed<'de, R>,
{
	fn deserialize_indexed<D>(
		index: &ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor<'a, R, T> {
			index: &'a ResourceBackwardIndex<R>,
			t: PhantomData<T>,
		}

		impl<'de, 'a, R, T> serde::de::Visitor<'de> for Visitor<'a, R, T>
		where
			T: DeserializeIndexed<'de, R>,
		{
			type Value = Vec<T>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a sequence")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let mut result = Vec::new();

				while let Some(item) = seq.next_element_seed(self.index.deserialize_seed())? {
					result.push(item)
				}

				Ok(result)
			}
		}

		deserializer.deserialize_seq(Visitor {
			index,
			t: PhantomData,
		})
	}
}

impl<'de, R, K, V> DeserializeIndexed<'de, R> for BTreeMap<K, V>
where
	K: Ord + DeserializeIndexed<'de, R>,
	V: DeserializeIndexed<'de, R>,
{
	fn deserialize_indexed<D>(
		index: &ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor<'a, R, K, V> {
			index: &'a ResourceBackwardIndex<R>,
			kv: PhantomData<(K, V)>,
		}

		impl<'de, 'a, R, K, V> serde::de::Visitor<'de> for Visitor<'a, R, K, V>
		where
			K: Ord + DeserializeIndexed<'de, R>,
			V: DeserializeIndexed<'de, R>,
		{
			type Value = BTreeMap<K, V>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a sequence")
			}

			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::MapAccess<'de>,
			{
				let mut result = BTreeMap::new();

				while let Some((key, value)) = map
					.next_entry_seed(self.index.deserialize_seed(), self.index.deserialize_seed())?
				{
					result.insert(key, value);
				}

				Ok(result)
			}
		}

		deserializer.deserialize_seq(Visitor {
			index,
			kv: PhantomData,
		})
	}
}

pub struct ResourceBackwardIndexSeed<'a, R, T> {
	index: &'a ResourceBackwardIndex<R>,
	t: PhantomData<T>,
}

impl<'a, R, T> ResourceBackwardIndexSeed<'a, R, T> {
	pub fn new(index: &'a ResourceBackwardIndex<R>) -> Self {
		Self {
			index,
			t: PhantomData,
		}
	}
}

impl<'a, 'de, R, T> serde::de::DeserializeSeed<'de> for ResourceBackwardIndexSeed<'a, R, T>
where
	T: DeserializeIndexed<'de, R>,
{
	type Value = T;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		T::deserialize_indexed(self.index, deserializer)
	}
}

impl<'a, 'de, R: Clone> serde::de::DeserializeSeed<'de> for &'a ResourceBackwardIndex<R> {
	type Value = R;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let index = u32::deserialize(deserializer)?;
		self.get(index)
			.cloned()
			.ok_or_else(|| serde::de::Error::custom("missing resource"))
	}
}

pub trait SerializeIndexed<R> {
	fn serialize_indexed<S>(
		&self,
		index: &ResourceForwardIndex<R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer;
}

impl<R, T> SerializeIndexed<R> for [T]
where
	T: SerializeIndexed<R>,
{
	fn serialize_indexed<S>(
		&self,
		index: &ResourceForwardIndex<R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut s = serializer.serialize_seq(Some(self.len()))?;

		for item in self {
			s.serialize_element(&Indexed::new(index, item))?;
		}

		s.end()
	}
}

pub struct Indexed<'a, R, T: ?Sized> {
	pub index: &'a ResourceForwardIndex<'a, R>,
	pub value: &'a T,
}

impl<'a, R, T: ?Sized> Indexed<'a, R, T> {
	pub fn new(index: &'a ResourceForwardIndex<R>, value: &'a T) -> Self {
		Self { index, value }
	}
}

impl<'a, R, T: ?Sized> serde::Serialize for Indexed<'a, R, T>
where
	T: SerializeIndexed<R>,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.value.serialize_indexed(&self.index, serializer)
	}
}
