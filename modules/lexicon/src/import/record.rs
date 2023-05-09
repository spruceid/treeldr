use iref::AsIri;
use rdf_types::{Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{import::sub_id, LexRecord};

use super::{nsid_name, Context, Item, OutputSubject, OutputTriple, Process};

impl<V: VocabularyMut> Process<V> for LexRecord {
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

		triples.push(Triple(
			id.clone(),
			vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
			Object::Literal(Literal::String(
				nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str()).to_string(),
			)),
		));

		if let Some(_) = self.key {
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
