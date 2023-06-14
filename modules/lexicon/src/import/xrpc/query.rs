use iref::AsIri;
use rdf_types::{literal, Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{
	import::{sub_id, OutputLiteralType},
	LexXrpcParametersNonPrimitiveProperty, LexXrpcParametersProperty, LexXrpcQuery,
};

use super::{
	super::{nsid_name, Context, Item, OutputSubject, OutputTriple, Process},
	process_xrpc_parameters,
};

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V> for LexXrpcQuery {
	fn process(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		stack: &mut Vec<Item<V>>,
		triples: &mut Vec<OutputTriple<V>>,
		_context: &Context,
		id: OutputSubject<V>,
	) where
		V::Iri: Clone,
		V::BlankId: Clone,
	{
		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::Rdf::Type.as_iri()),
			Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
		));

		let xsd_string = vocabulary.insert(vocab::Xsd::String.as_iri());
		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
			Object::Literal(vocabulary.insert_owned_literal(Literal::new(
				nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str()).to_string(),
				literal::Type::Any(xsd_string),
			))),
		));

		if let Some(desc) = self.description {
			let xsd_string = vocabulary.insert(vocab::Xsd::String.as_iri());
			triples.push(Triple(
				id.clone(),
				vocabulary.insert(vocab::Rdfs::Comment.as_iri()),
				Object::Literal(
					vocabulary
						.insert_owned_literal(Literal::new(desc, literal::Type::Any(xsd_string))),
				),
			));
		}

		let fields_id =
			process_xrpc_parameters(vocabulary, generator, stack, triples, &id, self.parameters);

		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::TreeLdr::Fields.as_iri()),
			Object::Id(fields_id),
		));

		if let Some(output) = self.output {
			let o_id = sub_id(vocabulary, &id, "output");
			stack.push(Item::XrpcBody(o_id, output))
		}
	}
}

impl<V: VocabularyMut> Process<V> for LexXrpcParametersProperty {
	fn process(
		self,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
		stack: &mut Vec<Item<V>>,
		_triples: &mut Vec<OutputTriple<V>>,
		_context: &Context,
		id: OutputSubject<V>,
	) where
		V::Iri: Clone,
		V::BlankId: Clone,
	{
		match self {
			LexXrpcParametersProperty::Primitive(p) => stack.push(Item::Primitive(id, p)),
			LexXrpcParametersProperty::NonPrimitive(n) => match n {
				LexXrpcParametersNonPrimitiveProperty::Array(a) => {
					stack.push(Item::PrimitiveArray(id, a))
				}
			},
		}
	}
}
