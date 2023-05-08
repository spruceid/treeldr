use contextual::{DisplayWithContext, WithContext};
use iref::{AsIri, Iri, IriBuf};
use rdf_types::{
	BlankIdVocabulary, Generator, Id, IriVocabulary, Literal, Object, Triple, Vocabulary,
	VocabularyMut,
};
use treeldr::vocab;

use crate::{
	LexBoolean, LexInteger, LexObject, LexPrimitive, LexPrimitiveArray, LexRef, LexRefUnion,
	LexRefVariant, LexString, LexUnknown, LexUserType, LexXrpcBody, LexXrpcBodySchema,
	LexXrpcParametersNonPrimitiveProperty, LexXrpcParametersProperty, LexXrpcQuery, LexiconDoc,
	ObjectNonPrimitiveProperty, ObjectProperty,
};

/// Checks if the given JSON document is a supported Lexicon document.
pub fn is_lexicon_document<M>(json: &json_syntax::Value<M>) -> bool {
	match json.as_object() {
		Some(object) => match object.get("lexicon").next() {
			Some(value) => match value.as_number() {
				Some(number) => number.as_str() == "1",
				None => false,
			},
			None => false,
		},
		None => false,
	}
}

pub type OutputSubject<V> = Id<<V as IriVocabulary>::Iri, <V as BlankIdVocabulary>::BlankId>;
pub type OutputPredicate<V> = <V as IriVocabulary>::Iri;
pub type OutputObject<V> = Object<OutputSubject<V>, Literal<String, <V as IriVocabulary>::Iri>>;
pub type OutputTriple<V> = Triple<OutputSubject<V>, OutputPredicate<V>, OutputObject<V>>;

trait RdfId<V: VocabularyMut> {
	fn rdf_id(&self, vocabulary: &mut V, namespace: Iri) -> OutputSubject<V>;
}

pub struct IntoTriples<'v, V: Vocabulary, G> {
	vocabulary: &'v mut V,
	generator: G,
	stack: Vec<Item<V>>,
	pending: Vec<OutputTriple<V>>,
}

impl<'v, V: Vocabulary, G> IntoTriples<'v, V, G> {
	pub fn new(doc: LexiconDoc, vocabulary: &'v mut V, generator: G) -> Self {
		Self {
			vocabulary,
			generator,
			stack: vec![Item::Doc(doc)],
			pending: Vec::new(),
		}
	}
}

impl<'v, V: VocabularyMut, G: Generator<V>> Iterator for IntoTriples<'v, V, G>
where
	V::Iri: Clone,
	V::BlankId: Clone,
	OutputTriple<V>: DisplayWithContext<V>,
{
	type Item = Triple<
		Id<V::Iri, V::BlankId>,
		V::Iri,
		Object<Id<V::Iri, V::BlankId>, Literal<String, V::Iri>>,
	>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(triple) = self.pending.pop() {
			// eprintln!("{} .", triple.with(&*self.vocabulary));
			return Some(triple);
		}

		while let Some(item) = self.stack.pop() {
			item.process(
				self.vocabulary,
				&mut self.generator,
				&mut self.stack,
				&mut self.pending,
			);

			if let Some(triple) = self.pending.pop() {
				// eprintln!("{} .", triple.with(&*self.vocabulary));
				return Some(triple);
			}
		}

		None
	}
}

pub enum Item<V: Vocabulary> {
	Doc(LexiconDoc),
	UserType(OutputSubject<V>, LexUserType),
	XrpcQuery(OutputSubject<V>, LexXrpcQuery),
	XrpcParametersProperty(OutputSubject<V>, LexXrpcParametersProperty),
	XrpcBody(OutputSubject<V>, LexXrpcBody),
	Primitive(OutputSubject<V>, LexPrimitive),
	PrimitiveArray(OutputSubject<V>, LexPrimitiveArray),
	RefVariant(OutputSubject<V>, LexRefVariant),
	Ref(OutputSubject<V>, LexRef),
	RefUnion(OutputSubject<V>, LexRefUnion),
	Boolean(OutputSubject<V>, LexBoolean),
	Integer(OutputSubject<V>, LexInteger),
	String(OutputSubject<V>, LexString),
	Object(OutputSubject<V>, LexObject),
	ObjectProperty(OutputSubject<V>, ObjectProperty),
	Unknown(OutputSubject<V>, LexUnknown),
}

impl<V: VocabularyMut> Item<V> {
	pub fn process(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		stack: &mut Vec<Item<V>>,
		triples: &mut Vec<OutputTriple<V>>,
	) where
		V::Iri: Clone,
		V::BlankId: Clone,
	{
		match self {
			Self::Doc(doc) => {
				if let Some(main) = doc.definitions.main {
					let iri = IriBuf::from_string(format!("lexicon:{}", doc.id)).unwrap();
					let id = Id::Iri(vocabulary.insert(iri.as_iri()));
					stack.push(Item::UserType(id, main.into()))
				}

				for (suffix, ty) in doc.definitions.other {
					let iri =
						IriBuf::from_string(format!("lexicon:{}.{}", doc.id, suffix)).unwrap();
					let id = Id::Iri(vocabulary.insert(iri.as_iri()));
					stack.push(Item::UserType(id, ty))
				}
			}
			Self::UserType(id, ty) => match ty {
				LexUserType::Record(_) => {
					log::warn!("records are not yet supported")
				}
				LexUserType::Query(q) => stack.push(Item::XrpcQuery(id, q)),
				LexUserType::Procedure(_) => {
					log::warn!("procedures are not yet supported")
				}
				LexUserType::Subscription(_) => {
					log::warn!("subscriptions are not yet supported")
				}
				LexUserType::Array(_) => {
					log::warn!("arrays are not yet supported")
				}
				LexUserType::Token(_) => {
					log::warn!("tokens are not yet supported")
				}
				LexUserType::Object(o) => stack.push(Item::Object(id, o)),
				LexUserType::Boolean(b) => stack.push(Item::Boolean(id, b)),
				LexUserType::Integer(i) => stack.push(Item::Integer(id, i)),
				LexUserType::String(s) => stack.push(Item::String(id, s)),
				LexUserType::Bytes(_) => {
					log::warn!("bytes are not yet supported")
				}
				LexUserType::CidLink(_) => {
					log::warn!("CID links are not yet supported")
				}
				LexUserType::Unknown(u) => stack.push(Item::Unknown(id, u)),
			},
			Self::XrpcQuery(id, q) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));

				let fields_id = match q.parameters {
					Some(params) => build_rdf_list(
						vocabulary,
						generator,
						triples,
						params.properties,
						|vocabulary, generator, triples, (name, p)| {
							let f_id = generator.next(vocabulary);

							triples.push(Triple(
								f_id.clone(),
								vocabulary.insert(vocab::Rdf::Type.as_iri()),
								Object::Id(Id::Iri(
									vocabulary.insert(vocab::TreeLdr::Field.as_iri()),
								)),
							));

							triples.push(Triple(
								f_id.clone(),
								vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
								Object::Literal(Literal::String(name.clone())),
							));

							let item_iri = IriBuf::new(&format!(
								"{}.{}",
								vocabulary.iri(id.as_iri().unwrap()).unwrap(),
								name
							))
							.unwrap();
							let item_id = Id::Iri(vocabulary.insert(item_iri.as_iri()));
							stack.push(Item::XrpcParametersProperty(item_id.clone(), p));

							let t_id = generator.next(vocabulary);
							triples.push(Triple(
								t_id.clone(),
								vocabulary.insert(vocab::Rdf::Type.as_iri()),
								Object::Id(Id::Iri(
									vocabulary.insert(vocab::TreeLdr::Layout.as_iri()),
								)),
							));

							if params.required.contains(&name) {
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
					),
					None => Id::Iri(vocabulary.insert(vocab::Rdf::Nil.as_iri())),
				};

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Fields.as_iri()),
					Object::Id(fields_id),
				));

				if let Some(output) = q.output {
					let o_iri = IriBuf::new(&format!(
						"{}.output",
						vocabulary.iri(id.as_iri().unwrap()).unwrap()
					))
					.unwrap();
					let o_id = Id::Iri(vocabulary.insert(o_iri.as_iri()));
					stack.push(Item::XrpcBody(o_id, output))
				}
			}
			Self::XrpcParametersProperty(id, p) => match p {
				LexXrpcParametersProperty::Primitive(p) => stack.push(Item::Primitive(id, p)),
				LexXrpcParametersProperty::NonPrimitive(n) => match n {
					LexXrpcParametersNonPrimitiveProperty::Array(a) => {
						stack.push(Item::PrimitiveArray(id, a))
					}
				},
			},
			Self::XrpcBody(id, b) => match b.schema {
				LexXrpcBodySchema::Object(o) => stack.push(Item::Object(id, o)),
				LexXrpcBodySchema::Ref(r) => stack.push(Item::RefVariant(id, r)),
			},
			Self::Primitive(id, p) => match p {
				LexPrimitive::Boolean(b) => stack.push(Item::Boolean(id, b)),
				LexPrimitive::Integer(i) => stack.push(Item::Integer(id, i)),
				LexPrimitive::String(s) => stack.push(Item::String(id, s)),
				LexPrimitive::Unknown(u) => stack.push(Item::Unknown(id, u)),
			},
			Self::PrimitiveArray(id, a) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));

				let item_iri = IriBuf::from_string(format!(
					"{}.items",
					vocabulary.iri(id.as_iri().unwrap()).unwrap()
				))
				.unwrap();
				let item_id = Id::Iri(vocabulary.insert(item_iri.as_iri()));
				stack.push(Item::Primitive(item_id.clone(), a.items));

				triples.push(Triple(
					id,
					vocabulary.insert(vocab::TreeLdr::Array.as_iri()),
					Object::Id(item_id),
				));
			}
			Self::RefVariant(id, r) => match r {
				LexRefVariant::Ref(r) => stack.push(Item::Ref(id, r)),
				LexRefVariant::Union(u) => stack.push(Item::RefUnion(id, u)),
			},
			Self::Ref(id, r) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));

				let iri = IriBuf::from_string(format!("lexicon:{}", r.ref_)).unwrap();

				triples.push(Triple(
					id,
					vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(iri.as_iri()))),
				));
			}
			Self::RefUnion(id, r) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));

				if r.closed.is_some() {
					log::warn!("ref union `closed` constraint not yet supported")
				}

				let variants_id = build_rdf_list(
					vocabulary,
					generator,
					triples,
					r.refs,
					|vocabulary, generator, triples, r| {
						let v_id = generator.next(vocabulary);

						triples.push(Triple(
							v_id.clone(),
							vocabulary.insert(vocab::Rdf::Type.as_iri()),
							Object::Id(Id::Iri(
								vocabulary.insert(vocab::TreeLdr::Variant.as_iri()),
							)),
						));

						triples.push(Triple(
							v_id.clone(),
							vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
							Object::Literal(Literal::String(nsid_name(&r).to_string())),
						));

						let format_iri = IriBuf::from_string(format!("lexicon:{}", r)).unwrap();
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
			Self::Object(id, o) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));

				if !o.nullable.is_empty() {
					log::warn!("object `nullable` constraint not yet supported")
				}

				let fields_id = build_rdf_list(
					vocabulary,
					generator,
					triples,
					o.properties,
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
							"{}.{}",
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

						if o.required.contains(&name) {
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
			Self::ObjectProperty(id, p) => match p {
				ObjectProperty::Ref(r) => stack.push(Item::RefVariant(id, r)),
				ObjectProperty::Primitive(p) => stack.push(Item::Primitive(id, p)),
				ObjectProperty::NonPrimitive(ObjectNonPrimitiveProperty::Array(_)) => {
					log::warn!("arrays are not yet supported")
				}
				ObjectProperty::NonPrimitive(ObjectNonPrimitiveProperty::Blob(_)) => {
					log::warn!("blobs are not yet supported")
				}
				ObjectProperty::Ipld(_) => {
					log::warn!("IPLD types are not yet supported")
				}
			},
			Self::Boolean(id, b) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));

				if b.const_.is_some() {
					log::warn!("boolean `const` constraint not yet supported")
				}

				if b.default.is_some() {
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
			Self::Integer(id, i) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));

				if i.const_.is_some() {
					log::warn!("integer `const` constraint not yet supported")
				}

				if i.default.is_some() {
					log::warn!("integer `default` constraint not yet supported")
				}

				if i.enum_.is_some() {
					log::warn!("integer `enum` constraint not yet supported")
				}

				if i.minimum.is_some() {
					log::warn!("integer `minimum` constraint not yet supported")
				}

				if i.maximum.is_some() {
					log::warn!("integer `maximum` constraint not yet supported")
				}

				triples.push(Triple(
					id,
					vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
					Object::Id(Id::Iri(
						vocabulary.insert(vocab::Primitive::Boolean.as_iri()),
					)),
				));
			}
			Self::String(id, s) => {
				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				triples.push(Triple(
					id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));

				if s.const_.is_some() {
					log::warn!("string `const` constraint not yet supported")
				}

				if s.default.is_some() {
					log::warn!("string `default` constraint not yet supported")
				}

				if s.enum_.is_some() {
					log::warn!("string `enum` constraint not yet supported")
				}

				if s.min_length.is_some() {
					log::warn!("string `min_length` constraint not yet supported")
				}

				if s.max_length.is_some() {
					log::warn!("string `max_length` constraint not yet supported")
				}

				if s.min_grapheme.is_some() {
					log::warn!("string `min_grapheme` constraint not yet supported")
				}

				if s.max_grapheme.is_some() {
					log::warn!("string `max_grapheme` constraint not yet supported")
				}

				if s.format.is_some() {
					log::warn!("string `format` constraint not yet supported")
				}

				triples.push(Triple(
					id,
					vocabulary.insert(vocab::TreeLdr::Alias.as_iri()),
					Object::Id(Id::Iri(
						vocabulary.insert(vocab::Primitive::Boolean.as_iri()),
					)),
				));
			}
			Self::Unknown(id, _) => {
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
						nsid_name(vocabulary.iri(id.as_iri().unwrap()).unwrap().as_str())
							.to_string(),
					)),
				));
			}
		}
	}
}

fn nsid_name(nsid: &str) -> &str {
	match nsid.rsplit_once('.') {
		Some((_, r)) => r,
		None => nsid,
	}
}

fn build_rdf_list<V: VocabularyMut, G: Generator<V>, I: IntoIterator>(
	vocabulary: &mut V,
	generator: &mut G,
	triples: &mut Vec<OutputTriple<V>>,
	items: I,
	mut f: impl FnMut(&mut V, &mut G, &mut Vec<OutputTriple<V>>, I::Item) -> OutputObject<V>,
) -> OutputSubject<V>
where
	I::IntoIter: DoubleEndedIterator,
	V::Iri: Clone,
	V::BlankId: Clone,
{
	let mut head = Id::Iri(vocabulary.insert(vocab::Rdf::Nil.as_iri()));

	for item in items.into_iter().rev() {
		let node = generator.next(vocabulary);

		triples.push(Triple(
			node.clone(),
			vocabulary.insert(vocab::Rdf::Type.as_iri()),
			Object::Id(Id::Iri(vocabulary.insert(vocab::Rdf::List.as_iri()))),
		));

		let first = f(vocabulary, generator, triples, item);

		triples.push(Triple(
			node.clone(),
			vocabulary.insert(vocab::Rdf::First.as_iri()),
			first,
		));

		triples.push(Triple(
			node.clone(),
			vocabulary.insert(vocab::Rdf::Rest.as_iri()),
			Object::Id(head),
		));

		head = node
	}

	head
}
