mod environment;

pub use environment::Environment;
use rdf_types::{
	InterpretationMut, ReverseTermInterpretationMut, TermInterpretationMut, VocabularyMut,
};

use crate::{RdfContextMut, RdfType};

pub enum Error {
	InvalidId(String),
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
	) -> Result<(), Error>;
}
