use iref::AsIri;
use rdf_types::{literal, Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{import::sub_id, LexRecord};

use super::{nsid_name, Context, Item, OutputLiteralType, OutputSubject, OutputTriple, Process};

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V> for LexRecord {
	fn process(
		self,
		vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
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

		if self.key.is_some() {
			log::warn!("records `key` constraint not yet supported");
		}

		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::TreeLdr::MapKey.as_iri()),
			Object::Id(Id::Iri(vocabulary.insert(vocab::Xsd::String.as_iri()))),
		));

		let record_id = sub_id(vocabulary, &id, "record");
		stack.push(Item::Object(record_id.clone(), self.record));

		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::TreeLdr::MapValue.as_iri()),
			Object::Id(record_id),
		));
	}
}
