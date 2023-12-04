use rdf_types::{
	Interpretation, InterpretationMut, ReverseTermInterpretationMut, TermInterpretationMut,
	VocabularyMut,
};
#[cfg(feature = "derive")]
pub use treeldr_derive::{DeserializeLd, SerializeLd};

#[doc(hidden)]
pub use iref;

#[doc(hidden)]
pub use rdf_types;

#[doc(hidden)]
pub use grdf;

mod datatypes;
mod environment;
mod pattern;
mod rdf;

pub use environment::Environment;
pub use pattern::Pattern;
pub use rdf::{RdfContext, RdfContextMut, RdfType};

pub enum SerializeError {
	InvalidId(String),
}

pub enum DeserializeError {
	// ...
}

pub trait SerializeLd<const N: usize, V, I>: Sized
where
	V: VocabularyMut<Value = String, Type = RdfType<V>>,
	I: InterpretationMut<V>
		+ TermInterpretationMut<V::Iri, V::BlankId, V::Literal>
		+ ReverseTermInterpretationMut<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
	I::Resource: Clone + Ord,
{
	fn serialize_ld_with(
		&self,
		rdf: &mut RdfContextMut<V, I>,
		inputs: &[I::Resource; N],
		current_graph: Option<&I::Resource>,
		output: &mut grdf::BTreeDataset<I::Resource>,
	) -> Result<(), SerializeError>;
}

pub trait DeserializeLd<const N: usize, V, I: Interpretation> {
	fn deserialize_ld<D>(dataset: &D, inputs: [I::Resource; N]) -> Result<(), DeserializeError>;
}
