use std::hash::Hash;

use rdf_types::{pattern::ResourceOrVar, Quad};
use serde_seeded::{DeserializeSeeded, SerializeSeeded};

use crate::{
	rdf::{
		serde::serde_indexed_resource, ResourceBackwardIndex, ResourceForwardIndex, VisitResources,
	},
	Value,
};

use super::Substitution;

pub type Graph<R = rdf_types::Term> = rdf_types::dataset::BTreeGraph<TermPattern<R>>;

pub type DatasetPattern<R = rdf_types::Term> = rdf_types::dataset::BTreeDataset<TermPattern<R>>;

/// A quad of patterns referencing their resources.
pub type PatternRefQuad<'p, R> = Quad<TermPattern<&'p R>>;

/// Pattern.
///
/// Either a resource identifier or a variable.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(R: Eq + Hash)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(R: Clone + Ord))
)]
pub enum TermPattern<R> {
	/// Resource.
	Resource(#[seeded(with(serde_indexed_resource))] R),

	/// Variable.
	Var(u32),
}

impl<R> TermPattern<R> {
	pub fn apply(&self, substitution: &Substitution<R>) -> Self
	where
		R: Clone,
	{
		match self {
			Self::Resource(r) => Self::Resource(r.clone()),
			Self::Var(x) => match substitution.get(*x).and_then(Value::as_resource) {
				Some(r) => Self::Resource(r.clone()),
				None => Self::Var(*x),
			},
		}
	}

	pub fn as_ref(&self) -> TermPattern<&R> {
		match self {
			Self::Resource(r) => TermPattern::Resource(r),
			Self::Var(x) => TermPattern::Var(*x),
		}
	}

	pub fn into_resource(self) -> Result<R, Self> {
		match self {
			Self::Resource(r) => Ok(r),
			other => Err(other),
		}
	}

	pub fn quad_into_resources(q: Quad<Self>) -> Result<Quad<R>, Quad<Self>> {
		match q.0.into_resource() {
			Ok(s) => match q.1.into_resource() {
				Ok(p) => match q.2.into_resource() {
					Ok(o) => match q.3 {
						Some(g) => match g.into_resource() {
							Ok(g) => Ok(Quad(s, p, o, Some(g))),
							Err(g) => Err(Quad(
								Self::Resource(s),
								Self::Resource(p),
								Self::Resource(o),
								Some(g),
							)),
						},
						None => Ok(Quad(s, p, o, None)),
					},
					Err(o) => Err(Quad(Self::Resource(s), Self::Resource(p), o, q.3)),
				},
				Err(p) => Err(Quad(Self::Resource(s), p, q.2, q.3)),
			},
			Err(s) => Err(Quad(s, q.1, q.2, q.3)),
		}
	}

	pub fn map<S>(self, f: impl FnOnce(R) -> S) -> TermPattern<S> {
		match self {
			Self::Resource(r) => TermPattern::Resource(f(r)),
			Self::Var(x) => TermPattern::Var(x),
		}
	}

	pub fn mapped<'a, S>(&'a self, f: impl FnOnce(&'a R) -> S) -> TermPattern<S> {
		match self {
			Self::Resource(r) => TermPattern::Resource(f(r)),
			Self::Var(x) => TermPattern::Var(*x),
		}
	}
}

impl<R> VisitResources<R> for TermPattern<R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		if let Self::Resource(r) = self {
			f(r)
		}
	}
}

impl<R> From<TermPattern<R>> for ResourceOrVar<R, u32> {
	fn from(value: TermPattern<R>) -> Self {
		match value {
			TermPattern::Resource(r) => Self::Resource(r),
			TermPattern::Var(x) => Self::Var(x),
		}
	}
}

pub mod serde_dataset_pattern {
	use super::TermPattern;
	use crate::{
		rdf::{ResourceBackwardIndex, ResourceForwardIndex},
		DatasetPattern,
	};
	use rdf_types::Quad;
	use serde::{ser::SerializeSeq, Deserializer, Serializer};
	use serde_seeded::{de::Seed, ser::Seeded, DeserializeSeeded, SerializeSeeded};
	use std::hash::Hash;

	pub struct IndexedQuad<T>(T);

	impl<R: Eq + Hash> SerializeSeeded<ResourceForwardIndex<R>> for IndexedQuad<Quad<&TermPattern<R>>> {
		fn serialize_seeded<S>(
			&self,
			seed: &ResourceForwardIndex<R>,
			serializer: S,
		) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			let mut s = serializer.serialize_seq(Some(4))?;

			s.serialize_element(&Seeded::new(seed, self.0 .0))?;
			s.serialize_element(&Seeded::new(seed, self.0 .1))?;
			s.serialize_element(&Seeded::new(seed, self.0 .2))?;
			s.serialize_element(&Seeded::new(seed, self.0 .3))?;

			s.end()
		}
	}

	impl<'de, R> DeserializeSeeded<'de, ResourceBackwardIndex<R>> for IndexedQuad<Quad<TermPattern<R>>>
	where
		R: Clone + Ord,
	{
		fn deserialize_seeded<D>(
			seed: &ResourceBackwardIndex<R>,
			deserializer: D,
		) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			struct Visitor<'seed, R>(&'seed ResourceBackwardIndex<R>);

			impl<'de, R> serde::de::Visitor<'de> for Visitor<'_, R>
			where
				R: Clone + Ord,
			{
				type Value = IndexedQuad<Quad<TermPattern<R>>>;

				fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
					write!(formatter, "a quad pattern")
				}

				fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
				where
					A: serde::de::SeqAccess<'de>,
				{
					let quad = IndexedQuad(Quad(
						seq.next_element_seed(Seed::new(self.0))?
							.ok_or_else(|| serde::de::Error::custom("missing quad subject"))?,
						seq.next_element_seed(Seed::new(self.0))?
							.ok_or_else(|| serde::de::Error::custom("missing quad predicate"))?,
						seq.next_element_seed(Seed::new(self.0))?
							.ok_or_else(|| serde::de::Error::custom("missing quad object"))?,
						seq.next_element_seed(Seed::new(self.0))?
							.ok_or_else(|| serde::de::Error::custom("missing quad graph"))?,
					));

					Ok(quad)
				}
			}

			deserializer.deserialize_seq(Visitor(seed))
		}
	}

	pub fn serialize_seeded<R, S>(
		dataset: &DatasetPattern<R>,
		index: &ResourceForwardIndex<R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		R: Eq + Hash,
		S: Serializer,
	{
		let mut s = serializer.serialize_seq(Some(dataset.len()))?;

		for pattern in dataset {
			s.serialize_element(&Seeded::new(index, IndexedQuad(pattern)))?;
		}

		s.end()
	}

	pub fn deserialize_seeded<'de, R, D>(
		index: &ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<DatasetPattern<R>, D::Error>
	where
		R: Clone + Ord,
		D: Deserializer<'de>,
	{
		struct Visitor<'seed, R>(&'seed ResourceBackwardIndex<R>);

		impl<'de, R> serde::de::Visitor<'de> for Visitor<'_, R>
		where
			R: Clone + Ord,
		{
			type Value = DatasetPattern<R>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a dataset pattern")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let mut result = DatasetPattern::new();

				while let Some(IndexedQuad(quad)) = seq.next_element_seed(Seed::new(self.0))? {
					result.insert(quad);
				}

				Ok(result)
			}
		}

		deserializer.deserialize_seq(Visitor(index))
	}
}
