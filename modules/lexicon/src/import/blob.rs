use iref::AsIri;
use rdf_types::{Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::LexBlob;

use super::{nsid_name, Context, Item, OutputSubject, OutputTriple, Process};

impl<V: VocabularyMut> Process<V> for LexBlob {
	fn process(
		self,
		vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
		_stack: &mut Vec<Item<V>>,
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

		if let Some(desc) = self.description {
			triples.push(Triple(
				id.clone(),
				vocabulary.insert(vocab::Rdfs::Comment.as_iri()),
				Object::Literal(Literal::String(desc)),
			));
		}

		if self.accept.is_some() {
			log::warn!("blob `accept` constraint not yet supported")
		}

		if self.max_size.is_some() {
			log::warn!("blob `max_size` constraint not yet supported")
		}

		triples.push(Triple(
			id,
			vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
			Object::Id(Id::Iri(vocabulary.insert(vocab::Primitive::Bytes.as_iri()))),
		));
	}
}
