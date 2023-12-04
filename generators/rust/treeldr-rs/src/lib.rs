use rdf_types::{Interpretation, InterpretationMut, VocabularyMut, ReverseTermInterpretationMut, TermInterpretationMut};
#[cfg(feature = "derive")]
pub use treeldr_derive::{SerializeLd, DeserializeLd};

#[doc(hidden)]
pub use iref;

#[doc(hidden)]
pub use rdf_types;

#[doc(hidden)]
pub use grdf;

mod rdf;
mod pattern;
mod environment;
mod datatypes;

pub use rdf::{RdfContext, RdfContextMut, RdfType};
pub use pattern::Pattern;
pub use environment::Environment;

pub enum SerializeError {
	InvalidId(String)
}

pub enum DeserializeError {
	// ...
}

pub trait SerializeLd<const N: usize, V, I>: Sized
where
	V: VocabularyMut<Value = String, Type = RdfType<V>>,
	I: InterpretationMut<V> + TermInterpretationMut<V::Iri, V::BlankId, V::Literal> + ReverseTermInterpretationMut<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	I::Resource: Clone + Ord
{
	fn serialize_ld_with(
		&self,
		rdf: &mut RdfContextMut<V, I>,
		inputs: &[I::Resource; N],
		current_graph: Option<&I::Resource>,
		output: &mut grdf::BTreeDataset<I::Resource>
	) -> Result<(), SerializeError>;
}

pub trait DeserializeLd<const N: usize, V, I: Interpretation> {
	fn deserialize_ld<D>(
		dataset: &D,
		inputs: [I::Resource; N]
	) -> Result<(), DeserializeError>;
}