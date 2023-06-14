use iref::AsIri;
use rdf_types::{literal, Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{LexRef, LexRefUnion, LexRefVariant};

use super::{
	build_rdf_list, nsid_name, Context, Item, OutputLiteralType, OutputSubject, OutputTriple,
	Process,
};

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V> for LexRefVariant {
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
			LexRefVariant::Ref(r) => stack.push(Item::Ref(id, r)),
			LexRefVariant::Union(u) => stack.push(Item::RefUnion(id, u)),
		}
	}
}

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V> for LexRef {
	fn process(
		self,
		vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
		_stack: &mut Vec<Item<V>>,
		triples: &mut Vec<OutputTriple<V>>,
		context: &Context,
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

		let iri = context.resolve_reference(&self.ref_);

		triples.push(Triple(
			id,
			vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
			Object::Id(Id::Iri(vocabulary.insert(iri.as_iri()))),
		));
	}
}

impl<V: VocabularyMut<Type = OutputLiteralType<V>, Value = String>> Process<V> for LexRefUnion {
	fn process(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		_stack: &mut Vec<Item<V>>,
		triples: &mut Vec<OutputTriple<V>>,
		context: &Context,
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

		if self.closed.is_some() {
			log::warn!("ref union `closed` constraint not yet supported")
		}

		let variants_id = build_rdf_list(
			vocabulary,
			generator,
			triples,
			self.refs,
			|vocabulary, generator, triples, r| {
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
						nsid_name(&r).to_string(),
						literal::Type::Any(xsd_string),
					))),
				));

				let format_iri = context.resolve_reference(&r);
				let format_id = Id::Iri(vocabulary.insert(format_iri.as_iri()));

				triples.push(Triple(
					v_id.clone(),
					vocabulary.insert(vocab::TreeLdr::Format.as_iri()),
					Object::Id(format_id),
				));

				Object::Id(v_id)
			},
		);

		triples.push(Triple(
			id,
			vocabulary.insert(vocab::TreeLdr::Enumeration.as_iri()),
			Object::Id(variants_id),
		));
	}
}
