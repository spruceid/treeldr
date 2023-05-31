use contextual::WithContext;
use iref::AsIri;
use rdf_types::{Generator, Id, Literal, Object, Triple, Vocabulary, VocabularyMut};
use treeldr::vocab;

use crate::{LexBoolean, LexInteger, LexPrimitive, LexString, LexUnknown};

use super::{
	build_rdf_list, nsid_name, Context, IntoItem, Item, OutputObject, OutputSubject, OutputTriple,
	Process,
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

		match self.default {
			Some(default_value) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::DerivedFrom.as_iri()),
					Object::Id(Id::Iri(
						vocabulary.insert(vocab::Primitive::Boolean.as_iri()),
					)),
				));

				triples.push(Triple(
					id,
					vocabulary.insert(vocab::TreeLdr::DefaultValue.as_iri()),
					Object::Literal(Literal::TypedString(
						default_value.to_string(),
						vocabulary.insert(vocab::Xsd::Boolean.as_iri()),
					)),
				));
			}
			None => triples.push(Triple(
				id,
				vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
				Object::Id(Id::Iri(
					vocabulary.insert(vocab::Primitive::Boolean.as_iri()),
				)),
			)),
		}
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

		if self.enum_.is_some() {
			log::warn!("integer `enum` constraint not yet supported")
		}

		let primitive = self.best_primitive();
		let (min, max) = self.bounds_constraints(primitive);

		let derived = self.default.is_some() || min.is_some() || max.is_some();

		if derived {
			triples.push(Triple(
				id.clone(),
				vocabulary.insert(vocab::TreeLdr::DerivedFrom.as_iri()),
				Object::Id(Id::Iri(vocabulary.insert(primitive.as_iri()))),
			));

			if let Some(default_value) = self.default {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::DefaultValue.as_iri()),
					Object::Literal(Literal::TypedString(
						default_value.to_string(),
						vocabulary.insert(
							primitive
								.natural_type()
								.unwrap()
								.id()
								.into_iri()
								.unwrap()
								.into_term()
								.unwrap()
								.iri(),
						),
					)),
				))
			}

			if min.is_some() || max.is_some() {
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
					id,
					vocabulary.insert(vocab::TreeLdr::WithRestrictions.as_iri()),
					Object::Id(constraints_id),
				));
			}
		} else {
			triples.push(Triple(
				id,
				vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
				Object::Id(Id::Iri(vocabulary.insert(primitive.as_iri()))),
			))
		}
	}
}

impl<V: VocabularyMut> Process<V> for LexString {
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

		if self.enum_.is_some() {
			log::warn!("string `enum` constraint not yet supported")
		}

		if self.format.is_some() {
			log::warn!("string `format` constraint not yet supported")
		}

		let mut restrictions: Vec<OutputObject<V>> = Vec::new();

		if let Some(value) = self.min_length {
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
				vocabulary.insert(vocab::TreeLdr::MinLength.as_iri()),
				Object::Literal(Literal::TypedString(
					value.to_string(),
					vocabulary.insert(vocab::Xsd::NonNegativeInteger.as_iri()),
				)),
			));

			restrictions.push(Object::Id(c_id))
		}

		if let Some(value) = self.max_length {
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
				vocabulary.insert(vocab::TreeLdr::MaxLength.as_iri()),
				Object::Literal(Literal::TypedString(
					value.to_string(),
					vocabulary.insert(vocab::Xsd::NonNegativeInteger.as_iri()),
				)),
			));

			restrictions.push(Object::Id(c_id))
		}

		if let Some(value) = self.min_grapheme {
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
				vocabulary.insert(vocab::TreeLdr::MinGrapheme.as_iri()),
				Object::Literal(Literal::TypedString(
					value.to_string(),
					vocabulary.insert(vocab::Xsd::NonNegativeInteger.as_iri()),
				)),
			));

			restrictions.push(Object::Id(c_id))
		}

		if let Some(value) = self.max_grapheme {
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
				vocabulary.insert(vocab::TreeLdr::MaxGrapheme.as_iri()),
				Object::Literal(Literal::TypedString(
					value.to_string(),
					vocabulary.insert(vocab::Xsd::NonNegativeInteger.as_iri()),
				)),
			));

			restrictions.push(Object::Id(c_id))
		}

		if restrictions.is_empty() && self.default.is_none() {
			triples.push(Triple(
				id,
				vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
				Object::Id(Id::Iri(
					vocabulary.insert(vocab::Primitive::String.as_iri()),
				)),
			));
		} else {
			triples.push(Triple(
				id.clone(),
				vocabulary.insert(vocab::TreeLdr::DerivedFrom.as_iri()),
				Object::Id(Id::Iri(
					vocabulary.insert(vocab::Primitive::String.as_iri()),
				)),
			));

			if let Some(default_value) = self.default {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::DefaultValue.as_iri()),
					Object::Literal(Literal::String(default_value)),
				))
			}

			if !restrictions.is_empty() {
				let restrictions_id = build_rdf_list(
					vocabulary,
					generator,
					triples,
					restrictions,
					|_, _, _, value| value,
				);

				triples.push(Triple(
					id,
					vocabulary.insert(vocab::TreeLdr::WithRestrictions.as_iri()),
					Object::Id(restrictions_id),
				));
			}
		}
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
