pub mod serde_indexed_resource {
	use std::hash::Hash;

	use serde::{Deserialize, Deserializer, Serialize, Serializer};

	use crate::rdf::{ResourceBackwardIndex, ResourceForwardIndex};

	pub fn serialize_seeded<R, S>(
		resource: &R,
		index: &ResourceForwardIndex<R>,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		R: Eq + Hash,
		S: Serializer,
	{
		index
			.get(resource)
			.ok_or_else(|| serde::ser::Error::custom("unindexed resource"))?
			.serialize(serializer)
	}

	pub fn deserialize_seeded<'de, R, D>(
		index: &ResourceBackwardIndex<R>,
		deserializer: D,
	) -> Result<R, D::Error>
	where
		R: Clone,
		D: Deserializer<'de>,
	{
		let i = u32::deserialize(deserializer)?;
		index
			.get(i)
			.cloned()
			.ok_or_else(|| serde::de::Error::custom("unknown resource index"))
	}
}
