use contextual::WithContext;
use iref::AsIri;
use rdf_types::{Generator, Id, Literal, Object, Triple, Vocabulary, VocabularyMut};
use treeldr::vocab;

use crate::{LexBoolean, LexInteger, LexPrimitive, LexString, LexUnknown};

use super::{
	build_rdf_list, nsid_name, Context, IntoItem, Item, OutputSubject, OutputTriple, Process,
};

impl<V: VocabularyMut> Process<V> for LexPrimitive {
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
			LexPrimitive::Boolean(b) => stack.push(Item::Boolean(id, b)),
			LexPrimitive::Integer(i) => stack.push(Item::Integer(id, i)),
			LexPrimitive::String(s) => stack.push(Item::String(id, s)),
			LexPrimitive::Unknown(u) => stack.push(Item::Unknown(id, u)),
		}
	}
}

impl<V: Vocabulary> IntoItem<V> for LexPrimitive {
	fn into_item(self, id: OutputSubject<V>) -> Item<V> {
		Item::Primitive(id, self)
	}
}

impl<V: VocabularyMut> Process<V> for LexBoolean {
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

		if self.const_.is_some() {
			log::warn!("boolean `const` constraint not yet supported")
		}

		if self.default.is_some() {
			log::warn!("boolean `default` constraint not yet supported")
		}

		triples.push(Triple(
			id,
			vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
			Object::Id(Id::Iri(
				vocabulary.insert(vocab::Primitive::Boolean.as_iri()),
			)),
		));
	}
}

impl<V: VocabularyMut> Process<V> for LexInteger {
	fn process(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
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

		if self.const_.is_some() {
			log::warn!("integer `const` constraint not yet supported")
		}

		if self.default.is_some() {
			log::warn!("integer `default` constraint not yet supported")
		}

		if self.enum_.is_some() {
			log::warn!("integer `enum` constraint not yet supported")
		}

		let primitive = self.best_primitive();
		match self.bounds_constraints(primitive) {
			(None, None) => {
				triples.push(Triple(
					id,
					vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(primitive.as_iri()))),
				));
			}
			(min, max) => {
				let constraits = min
					.into_iter()
					.map(|m| (vocab::TreeLdr::InclusiveMinimum, m))
					.chain(
						max.into_iter()
							.map(|m| (vocab::TreeLdr::InclusiveMaximum, m)),
					);

				let constraints_id = build_rdf_list(
					vocabulary,
					generator,
					triples,
					constraits,
					|vocabulary, generator, triples, (prop, value)| {
						let c_id = generator.next(vocabulary);

						triples.push(Triple(
							c_id.clone(),
							vocabulary.insert(vocab::Rdf::Type.as_iri()),
							Object::Id(Id::Iri(
								vocabulary.insert(vocab::TreeLdr::LayoutRestriction.as_iri()),
							)),
						));

						triples.push(Triple(
							c_id.clone(),
							vocabulary.insert(prop.as_iri()),
							Object::Literal(Literal::TypedString(
								value.to_string(),
								vocabulary.insert(primitive.natural_type_term().unwrap().as_iri()),
							)),
						));

						Object::Id(c_id)
					},
				);

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::DerivedFrom.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(primitive.as_iri()))),
				));

				triples.push(Triple(
					id,
					vocabulary.insert(vocab::TreeLdr::WithRestrictions.as_iri()),
					Object::Id(constraints_id),
				));
			}
		}
	}
}

impl<V: VocabularyMut> Process<V> for LexString {
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

		if self.const_.is_some() {
			log::warn!("string `const` constraint not yet supported")
		}

		if self.default.is_some() {
			log::warn!("string `default` constraint not yet supported")
		}

		if self.enum_.is_some() {
			log::warn!("string `enum` constraint not yet supported")
		}

		if self.min_length.is_some() {
			log::warn!("string `min_length` constraint not yet supported")
		}

		if self.max_length.is_some() {
			log::warn!("string `max_length` constraint not yet supported")
		}

		if self.min_grapheme.is_some() {
			log::warn!("string `min_grapheme` constraint not yet supported")
		}

		if self.max_grapheme.is_some() {
			log::warn!("string `max_grapheme` constraint not yet supported")
		}

		if self.format.is_some() {
			log::warn!("string `format` constraint not yet supported")
		}

		triples.push(Triple(
			id,
			vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
			Object::Id(Id::Iri(
				vocabulary.insert(vocab::Primitive::String.as_iri()),
			)),
		));
	}
}

impl<V: VocabularyMut> Process<V> for LexUnknown {
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
		log::warn!("unknown user type {}", id.with(&*vocabulary));
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
	}
}
