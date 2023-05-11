use iref::AsIri;
use rdf_types::{Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{import::sub_id, LexXrpcSubscription, LexXrpcSubscriptionMessage};

use super::{
	super::{build_rdf_list, nsid_name, Context, Item, OutputSubject, OutputTriple, Process},
	process_xrpc_parameters,
};

impl<V: VocabularyMut> Process<V> for LexXrpcSubscription {
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

		if let Some(message) = self.message {
			let msg_id = sub_id(vocabulary, &id, "message");
			stack.push(Item::XrpcSubscriptionMessage(msg_id, message))
		}

		if !self.errors.is_empty() {
			let error_id = sub_id(vocabulary, &id, "error");

			triples.push(Triple(
				error_id.clone(),
				vocabulary.insert(vocab::Rdf::Type.as_iri()),
				Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
			));

			triples.push(Triple(
				error_id.clone(),
				vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
				Object::Literal(Literal::String("error".to_string())),
			));

			let variants_id = build_rdf_list(
				vocabulary,
				generator,
				triples,
				self.errors,
				|vocabulary, generator, triples, e| {
					let v_id = generator.next(vocabulary);

					triples.push(Triple(
						v_id.clone(),
						vocabulary.insert(vocab::Rdf::Type.as_iri()),
						Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Variant.as_iri()))),
					));

					triples.push(Triple(
						v_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
						Object::Literal(Literal::String(e.name)),
					));

					Object::Id(v_id)
				},
			);

			triples.push(Triple(
				error_id,
				vocabulary.insert(vocab::TreeLdr::Enumeration.as_iri()),
				Object::Id(variants_id),
			));
		}

		if !self.infos.is_empty() {
			let info_id = sub_id(vocabulary, &id, "info");

			triples.push(Triple(
				info_id.clone(),
				vocabulary.insert(vocab::Rdf::Type.as_iri()),
				Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
			));

			triples.push(Triple(
				info_id.clone(),
				vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
				Object::Literal(Literal::String("info".to_string())),
			));

			let variants_id = build_rdf_list(
				vocabulary,
				generator,
				triples,
				self.infos,
				|vocabulary, generator, triples, i| {
					let v_id = generator.next(vocabulary);

					triples.push(Triple(
						v_id.clone(),
						vocabulary.insert(vocab::Rdf::Type.as_iri()),
						Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Variant.as_iri()))),
					));

					triples.push(Triple(
						v_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
						Object::Literal(Literal::String(i.name)),
					));

					Object::Id(v_id)
				},
			);

			triples.push(Triple(
				info_id,
				vocabulary.insert(vocab::TreeLdr::Enumeration.as_iri()),
				Object::Id(variants_id),
			));
		}

		let fields_id =
			process_xrpc_parameters(vocabulary, generator, stack, triples, &id, self.parameters);

		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::TreeLdr::Fields.as_iri()),
			Object::Id(fields_id),
		));
	}
}

impl<V: VocabularyMut> Process<V> for LexXrpcSubscriptionMessage {
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
		self.schema
			.process(vocabulary, generator, stack, triples, context, id)
	}
}
