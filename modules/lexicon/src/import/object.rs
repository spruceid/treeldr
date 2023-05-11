use iref::{AsIri, IriBuf};
use rdf_types::{Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::{LexObject, ObjectNonPrimitiveProperty, ObjectProperty};

use super::{build_rdf_list, nsid_name, Context, Item, OutputSubject, OutputTriple, Process};

impl<V: VocabularyMut> Process<V> for LexObject {
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

		if !self.nullable.is_empty() {
			log::warn!("object `nullable` constraint not yet supported")
		}

		let fields_id = build_rdf_list(
			vocabulary,
			generator,
			triples,
			self.properties,
			|vocabulary, generator, triples, (name, prop)| {
				let f_id = generator.next(vocabulary);

				triples.push(Triple(
					f_id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Field.as_iri()))),
				));

				triples.push(Triple(
					f_id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(name.clone())),
				));

				let item_iri = IriBuf::from_string(format!(
					"{}/{}",
					vocabulary.iri(id.as_iri().unwrap()).unwrap(),
					name
				))
				.unwrap();
				let item_id = Id::Iri(vocabulary.insert(item_iri.as_iri()));
				stack.push(Item::ObjectProperty(item_id.clone(), prop));

				let t_id = generator.next(vocabulary);
				triples.push(Triple(
					t_id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				if self.required.contains(&name) {
					triples.push(Triple(
						t_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Required.as_iri()),
						Object::Id(item_id),
					));
				} else {
					triples.push(Triple(
						t_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Option.as_iri()),
						Object::Id(item_id),
					));
				};

				triples.push(Triple(
					f_id.clone(),
					vocabulary.insert(vocab::TreeLdr::Format.as_iri()),
					Object::Id(t_id),
				));

				Object::Id(f_id)
			},
		);

		triples.push(Triple(
			id,
			vocabulary.insert(vocab::TreeLdr::Fields.as_iri()),
			Object::Id(fields_id),
		));
	}
}

impl<V: VocabularyMut> Process<V> for ObjectProperty {
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
		match self {
			ObjectProperty::Ref(r) => r.process(vocabulary, generator, stack, triples, context, id),
			ObjectProperty::Primitive(p) => {
				p.process(vocabulary, generator, stack, triples, context, id)
			}
			ObjectProperty::NonPrimitive(ObjectNonPrimitiveProperty::Array(a)) => {
				a.process(vocabulary, generator, stack, triples, context, id)
			}
			ObjectProperty::NonPrimitive(ObjectNonPrimitiveProperty::Blob(b)) => {
				b.process(vocabulary, generator, stack, triples, context, id)
			}
			ObjectProperty::Ipld(i) => {
				i.process(vocabulary, generator, stack, triples, context, id)
			}
		}
	}
}
