use iref::AsIri;
use rdf_types::{literal, Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{
	import::{nsid_name, OutputLiteralType},
	LexBytes,
};

use super::{Context, Item, OutputSubject, OutputTriple, Process};

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V> for LexBytes {
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

		if self.min_size.is_some() {
			log::warn!("bytes `min_size` constraint not yet supported")
		}

		if self.max_size.is_some() {
			log::warn!("bytes `max_size` constraint not yet supported")
		}

		triples.push(Triple(
			id,
			vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
			Object::Id(Id::Iri(
				vocabulary.insert(vocab::Primitive::BytesBuf.as_iri()),
			)),
		));
	}
}
