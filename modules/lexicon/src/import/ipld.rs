use rdf_types::{Generator, VocabularyMut};

use crate::LexIpldType;

use super::{Context, Item, OutputSubject, OutputTriple, Process};

mod bytes;
mod cid_link;

impl<V: VocabularyMut> Process<V> for LexIpldType {
	fn process(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		stack: &mut Vec<Item<V>>,
		triples: &mut Vec<OutputTriple<V>>,
		context: &Context,
		id: OutputSubject<V>,
	) where
		V::Iri: Clone,
		V::BlankId: Clone,
	{
		match self {
			Self::Bytes(b) => b.process(vocabulary, generator, stack, triples, context, id),
			Self::CidLink(l) => l.process(vocabulary, generator, stack, triples, context, id),
		}
	}
}
