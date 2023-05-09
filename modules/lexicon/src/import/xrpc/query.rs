use iref::{AsIri, IriBuf};
use rdf_types::{Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{
	LexXrpcBody, LexXrpcBodySchema, LexXrpcParametersNonPrimitiveProperty,
	LexXrpcParametersProperty, LexXrpcQuery,
};

use super::super::{
	build_rdf_list, nsid_name, Context, Item, OutputSubject, OutputTriple, Process,
};

impl<V: VocabularyMut> Process<V> for LexXrpcQuery {
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

		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
			Object::Literal(Literal::String(
				nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str()).to_string(),
			)),
		));

		let fields_id = match self.parameters {
			Some(params) => build_rdf_list(
				vocabulary,
				generator,
				triples,
				params.properties,
				|vocabulary, generator, triples, (name, p)| {
					let f_id = generator.next(vocabulary);

					triples.push(Triple(
						f_id.clone(),
						vocabulary.insert(vocab::Rdf::Type.as_iri()),
						Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Field.as_iri()))),
					));

					triples.push(Triple(
						f_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
						Object::Literal(Literal::String(name.clone())),
					));

					let item_iri = IriBuf::new(&format!(
						"{}/{}",
						vocabulary.iri(id.as_iri().unwrap()).unwrap(),
						name
					))
					.unwrap();
					let item_id = Id::Iri(vocabulary.insert(item_iri.as_iri()));
					stack.push(Item::XrpcParametersProperty(item_id.clone(), p));

					let t_id = generator.next(vocabulary);
					triples.push(Triple(
						t_id.clone(),
						vocabulary.insert(vocab::Rdf::Type.as_iri()),
						Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
					));

					if params.required.contains(&name) {
						triples.push(Triple(
							t_id.clone(),
							vocabulary.insert(vocab::TreeLdr::Required.as_iri()),
							Object::Id(item_id),
						));
					} else {
						triples.push(Triple(
							t_id.clone(),
							vocabulary.insert(vocab::TreeLdr::Option.as_iri()),
							Object::Id(item_id),
						));
					};

					triples.push(Triple(
						f_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Format.as_iri()),
						Object::Id(t_id),
					));

					Object::Id(f_id)
				},
			),
			None => Id::Iri(vocabulary.insert(vocab::Rdf::Nil.as_iri())),
		};

		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::TreeLdr::Fields.as_iri()),
			Object::Id(fields_id),
		));

		if let Some(output) = self.output {
			let o_iri = IriBuf::new(&format!(
				"{}/output",
				vocabulary.iri(id.as_iri().unwrap()).unwrap()
			))
			.unwrap();
			let o_id = Id::Iri(vocabulary.insert(o_iri.as_iri()));
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

impl<V: VocabularyMut> Process<V> for LexXrpcBody {
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
		match self.schema {
			LexXrpcBodySchema::Object(o) => stack.push(Item::Object(id, o)),
			LexXrpcBodySchema::Ref(r) => stack.push(Item::RefVariant(id, r)),
		}
	}
}
