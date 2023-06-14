use iref::AsIri;
use rdf_types::{literal, Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{
	import::{sub_id, OutputLiteralType},
	LexXrpcSubscription, LexXrpcSubscriptionMessage,
};

use super::{
	super::{build_rdf_list, nsid_name, Context, Item, OutputSubject, OutputTriple, Process},
	process_xrpc_parameters,
};

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V>
	for LexXrpcSubscription
{
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

			let xsd_string = vocabulary.insert(vocab::Xsd::String.as_iri());
			triples.push(Triple(
				error_id.clone(),
				vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
				Object::Literal(vocabulary.insert_owned_literal(Literal::new(
					"error".to_string(),
					literal::Type::Any(xsd_string),
				))),
			));

			let xsd_string = vocabulary.insert(vocab::Xsd::String.as_iri());
			triples.push(Triple(
				error_id.clone(),
				vocabulary.insert(vocab::Rdfs::Comment.as_iri()),
				Object::Literal(vocabulary.insert_owned_literal(Literal::new(
					format!(
						"Errors of <{}>.",
						vocabulary.iri(id.as_iri().unwrap()).unwrap()
					),
					literal::Type::Any(xsd_string),
				))),
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

					let xsd_string = vocabulary.insert(vocab::Xsd::String.as_iri());
					triples.push(Triple(
						v_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
						Object::Literal(vocabulary.insert_owned_literal(Literal::new(
							e.name,
							literal::Type::Any(xsd_string),
						))),
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

			let xsd_string = vocabulary.insert(vocab::Xsd::String.as_iri());
			triples.push(Triple(
				info_id.clone(),
				vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
				Object::Literal(vocabulary.insert_owned_literal(Literal::new(
					"info".to_string(),
					literal::Type::Any(xsd_string),
				))),
			));

			let xsd_string = vocabulary.insert(vocab::Xsd::String.as_iri());
			triples.push(Triple(
				info_id.clone(),
				vocabulary.insert(vocab::Rdfs::Comment.as_iri()),
				Object::Literal(vocabulary.insert_owned_literal(Literal::new(
					format!(
						"Infos of <{}>.",
						vocabulary.iri(id.as_iri().unwrap()).unwrap()
					),
					literal::Type::Any(xsd_string),
				))),
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

					let xsd_string = vocabulary.insert(vocab::Xsd::String.as_iri());
					triples.push(Triple(
						v_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
						Object::Literal(vocabulary.insert_owned_literal(Literal::new(
							i.name,
							literal::Type::Any(xsd_string),
						))),
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

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V>
	for LexXrpcSubscriptionMessage
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
		self.schema
			.process(vocabulary, generator, stack, triples, context, id)
	}
}
