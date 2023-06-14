use rdf_types::{Generator, VocabularyMut};

use crate::{
	import::{sub_id, OutputLiteralType},
	LexXrpcProcedure,
};

use super::super::{Context, Item, OutputSubject, OutputTriple, Process};

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V>
	for LexXrpcProcedure
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
		for (name, p) in self.parameters {
			let p_id = sub_id(vocabulary, &id, &name);
			stack.push(Item::Primitive(p_id, p));
		}

		if let Some(output) = self.output {
			let output_id = sub_id(vocabulary, &id, "output");
			stack.push(Item::XrpcBody(output_id, output));
		}

		if let Some(input) = self.input {
			input.process(vocabulary, generator, stack, triples, context, id)
		}
	}
}
