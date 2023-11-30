#[cfg(feature = "derive")]
pub use treeldr_derive::{Serialize, Deserialize};

pub enum SerializeError {
	// ...
}

pub enum DeserializeError {
	// ...
}

pub trait SerializeLd<const N: usize, R>: Sized {
	fn serialize_ld(
		&self,
		inputs: [R; N]
	) -> Result<grdf::BTreeDataset<R>, SerializeError>;
}

pub trait DeserializeLd<const N: usize, R> {
	fn deserialize_ld<D>(
		dataset: &D,
		inputs: [R; N]
	) -> Result<(), DeserializeError>;
}