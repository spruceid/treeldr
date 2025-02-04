use core::fmt;
use educe::Educe;
use serde_seeded::{DeserializeSeeded, SerializeSeeded};
use std::{cmp::Ordering, collections::BTreeSet, hash::Hash};

use crate::{
	display::DisplaySignature,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
};

/// Resource type.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe, SerializeSeeded, DeserializeSeeded,
)]
#[educe(Default)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(R: Eq + Hash)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(R: Clone + Ord))
)]
pub struct Resource<R> {
	#[seeded(
		default,
		skip_serializing_if(Option::is_none),
		with(serde_indexed_resource_set)
	)]
	set: Option<BTreeSet<R>>,
}

impl<R: Ord> Resource<R> {
	pub fn singleton(r: R) -> Self {
		let mut set = BTreeSet::new();
		set.insert(r);

		Self { set: Some(set) }
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (&self.set, &other.set) {
			(Some(a), Some(b)) => {
				if a == b {
					Some(Ordering::Equal)
				} else if a.is_superset(b) {
					Some(Ordering::Greater)
				} else if a.is_subset(b) {
					Some(Ordering::Less)
				} else {
					None
				}
			}
			(None, Some(_)) => Some(Ordering::Greater),
			(Some(_), None) => Some(Ordering::Less),
			(None, None) => Some(Ordering::Equal),
		}
	}

	pub fn contains(&self, resource: &R) -> bool {
		match &self.set {
			Some(set) => set.contains(resource),
			None => true,
		}
	}
}

impl<R: Clone> Resource<R> {
	pub fn to_singleton_value(&self) -> Option<R> {
		if let Some(set) = &self.set {
			if set.len() == 1 {
				return set.iter().next().cloned();
			}
		}

		None
	}
}

impl<R> VisitResources<R> for Resource<R> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		if let Some(set) = &self.set {
			for r in set {
				f(r)
			}
		}
	}
}

impl<P, R: fmt::Display> DisplaySignature<P, R> for Resource<R> {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<
			impl crate::Registry<PackageId = P, Resource = R>,
		>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match &self.set {
			Some(set) => {
				write!(f, "(any_of")?;

				for r in set {
					write!(f, " {r}")?;
				}

				write!(f, ")")
			}
			None => {
				write!(f, "resource")
			}
		}
	}
}

mod serde_indexed_resource_set {
	use std::{collections::BTreeSet, hash::Hash};

	use serde::{ser::SerializeSeq, Deserializer, Serializer};

	use crate::rdf::{ResourceBackwardIndex, ResourceForwardIndex};

	pub fn serialize_seeded<R, S>(
		set: &Option<BTreeSet<R>>,
		index: &ResourceForwardIndex<R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		R: Eq + Hash,
		S: Serializer,
	{
		let mut s =
			serializer.serialize_seq(Some(set.as_ref().map(BTreeSet::len).unwrap_or_default()))?;

		if let Some(set) = set.as_ref() {
			for r in set {
				let i = index
					.get(r)
					.ok_or_else(|| serde::ser::Error::custom("unindexed resource"))?;
				s.serialize_element(&i)?;
			}
		}

		s.end()
	}

	pub fn deserialize_seeded<'de, R, D>(
		index: &ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<Option<BTreeSet<R>>, D::Error>
	where
		R: Clone + Ord,
		D: Deserializer<'de>,
	{
		struct Visitor<'seed, R>(&'seed ResourceBackwardIndex<R>);

		impl<'de, 'seed, R> serde::de::Visitor<'de> for Visitor<'seed, R>
		where
			R: Clone + Ord,
		{
			type Value = BTreeSet<R>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a set of resources")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let mut result = BTreeSet::new();

				while let Some(i) = seq.next_element()? {
					let resource = self
						.0
						.get(i)
						.ok_or_else(|| serde::de::Error::custom("unknown resource index"))?;
					result.insert(resource.clone());
				}

				Ok(result)
			}
		}

		deserializer.deserialize_seq(Visitor(index)).map(Some)
	}
}
