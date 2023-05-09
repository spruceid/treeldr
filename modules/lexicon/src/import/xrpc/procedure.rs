use rdf_types::{Generator, VocabularyMut};

use crate::LexXrpcProcedure;

use super::super::{Context, Item, OutputSubject, OutputTriple, Process};

impl<V: VocabularyMut> Process<V> for LexXrpcProcedure {
	fn process(
		self,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
		_stack: &mut Vec<Item<V>>,
		_triples: &mut Vec<OutputTriple<V>>,
		_context: &Context,
		_id: OutputSubject<V>,
	) where
		V::Iri: Clone,
		V::BlankId: Clone,
	{
		log::warn!("XRPC procedure not yet supported");
	}
}
