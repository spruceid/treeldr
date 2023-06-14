use iref::{AsIri, IriBuf};
use rdf_types::{literal, Generator, Id, Literal, Object, Triple, Vocabulary, VocabularyMut};
use treeldr::vocab;

use crate::{ArrayItem, ArrayNonPrimitiveItem, LexArray};

use super::{
	nsid_name, Context, IntoItem, Item, OutputLiteralType, OutputSubject, OutputTriple, Process,
};

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>, T: IntoItem<V>> Process<V>
	for LexArray<T>
{
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

		if self.min_length.is_some() {
			log::warn!("array `maxLength` constraint not yet supported")
		}

		if self.max_length.is_some() {
			log::warn!("array `minLength` constraint not yet supported")
		}

		let item_iri = IriBuf::from_string(format!(
			"{}/items",
			vocabulary.iri(id.as_iri().unwrap()).unwrap()
		))
		.unwrap();
		let item_id = Id::Iri(vocabulary.insert(item_iri.as_iri()));
		stack.push(self.items.into_item(item_id.clone()));

		triples.push(Triple(
			id,
			vocabulary.insert(vocab::TreeLdr::Array.as_iri()),
			Object::Id(item_id),
		));
	}
}

impl<V: Vocabulary> IntoItem<V> for ArrayItem {
	fn into_item(self, id: OutputSubject<V>) -> Item<V> {
		match self {
			Self::Ref(r) => Item::RefVariant(id, r),
			Self::Primitive(p) => Item::Primitive(id, p),
			Self::Ipld(i) => Item::Ipld(id, i),
			Self::NonPrimitive(n) => n.into_item(id),
		}
	}
}

impl<V: Vocabulary> IntoItem<V> for ArrayNonPrimitiveItem {
	fn into_item(self, id: OutputSubject<V>) -> Item<V> {
		match self {
			Self::Blob(b) => Item::Blob(id, b),
		}
	}
}
