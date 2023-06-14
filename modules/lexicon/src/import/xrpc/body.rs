use rdf_types::{Generator, VocabularyMut};

use crate::{import::OutputLiteralType, LexXrpcBody, LexXrpcBodySchema};

use super::super::{Context, Item, OutputSubject, OutputTriple, Process};

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V> for LexXrpcBody {
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
		if let Some(schema) = self.schema {
			schema.process(vocabulary, generator, stack, triples, context, id)
		}
	}
}

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V>
	for LexXrpcBodySchema
{
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
			Self::Ref(r) => r.process(vocabulary, generator, stack, triples, context, id),
			Self::Object(o) => o.process(vocabulary, generator, stack, triples, context, id),
		}
	}
}
