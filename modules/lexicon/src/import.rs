use contextual::DisplayWithContext;
use iref::{AsIri, Iri, IriBuf};
use rdf_types::{
	BlankIdVocabulary, Generator, Id, IriVocabulary, Literal, Object, Triple, Vocabulary,
	VocabularyMut,
};
use treeldr::vocab;

use crate::{
	LexAnyUserType, LexBlob, LexBoolean, LexInteger, LexIpldType, LexObject, LexPrimitive,
	LexPrimitiveArray, LexRef, LexRefUnion, LexRefVariant, LexString, LexUnknown, LexXrpcBody,
	LexXrpcParametersProperty, LexXrpcSubscriptionMessage, LexiconDoc, Nsid, ObjectProperty,
};

mod array;
mod blob;
mod ipld;
mod object;
mod primitive;
mod record;
mod reference;
mod token;
mod xrpc;

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

struct Context {
	base_iri: IriBuf,
}

impl Context {
	fn resolve_reference(&self, r: &str) -> IriBuf {
		match r.split_once('#') {
			Some((prefix, fragment)) => {
				if prefix.is_empty() {
					let mut iri = self.base_iri.clone();
					iri.path_mut().push(fragment.try_into().unwrap());
					iri
				} else {
					let mut iri = Nsid::new(prefix).unwrap().as_iri();
					iri.path_mut().push(fragment.try_into().unwrap());
					iri
				}
			}
			None => Nsid::new(r).unwrap().as_iri(),
		}
	}
}

pub struct IntoTriples<'v, V: Vocabulary, G> {
	vocabulary: &'v mut V,
	generator: G,
	stack: Vec<Item<V>>,
	pending: Vec<OutputTriple<V>>,
	context: Context,
}

impl<'v, V: Vocabulary, G> IntoTriples<'v, V, G> {
	pub fn new(doc: LexiconDoc, vocabulary: &'v mut V, generator: G) -> Self {
		let base_iri = doc.id.as_iri();

		Self {
			vocabulary,
			generator,
			stack: vec![Item::Doc(doc)],
			pending: Vec::new(),
			context: Context { base_iri },
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
				&self.context,
			);

			if let Some(triple) = self.pending.pop() {
				// eprintln!("{} .", triple.with(&*self.vocabulary));
				return Some(triple);
			}
		}

		None
	}
}

enum Item<V: Vocabulary> {
	Doc(LexiconDoc),
	UserType(OutputSubject<V>, LexAnyUserType),
	XrpcParametersProperty(OutputSubject<V>, LexXrpcParametersProperty),
	XrpcBody(OutputSubject<V>, LexXrpcBody),
	XrpcSubscriptionMessage(OutputSubject<V>, LexXrpcSubscriptionMessage),
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
	Blob(OutputSubject<V>, LexBlob),
	Ipld(OutputSubject<V>, LexIpldType),
	Unknown(OutputSubject<V>, LexUnknown),
}

trait IntoItem<V: Vocabulary> {
	fn into_item(self, id: OutputSubject<V>) -> Item<V>;
}

trait Process<V: VocabularyMut> {
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
		V::BlankId: Clone;
}

impl<V: VocabularyMut> Item<V> {
	fn process(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		stack: &mut Vec<Item<V>>,
		triples: &mut Vec<OutputTriple<V>>,
		context: &Context,
	) where
		V::Iri: Clone,
		V::BlankId: Clone,
	{
		match self {
			Self::Doc(doc) => {
				if let Some(main) = doc.definitions.main {
					let iri = doc.id.as_iri();
					let id = Id::Iri(vocabulary.insert(iri.as_iri()));
					stack.push(Item::UserType(id, main))
				}

				for (suffix, ty) in doc.definitions.other {
					let iri =
						IriBuf::from_string(format!("{}/{}", doc.id.as_iri(), suffix)).unwrap();
					let id = Id::Iri(vocabulary.insert(iri.as_iri()));
					stack.push(Item::UserType(id, ty.into()))
				}
			}
			Self::UserType(id, ty) => {
				ty.process(vocabulary, generator, stack, triples, context, id)
			}
			Self::XrpcParametersProperty(id, p) => {
				p.process(vocabulary, generator, stack, triples, context, id)
			}
			Self::XrpcBody(id, b) => b.process(vocabulary, generator, stack, triples, context, id),
			Self::XrpcSubscriptionMessage(id, m) => {
				m.process(vocabulary, generator, stack, triples, context, id)
			}
			Self::Primitive(id, p) => p.process(vocabulary, generator, stack, triples, context, id),
			Self::PrimitiveArray(id, a) => {
				a.process(vocabulary, generator, stack, triples, context, id)
			}
			Self::RefVariant(id, r) => {
				r.process(vocabulary, generator, stack, triples, context, id)
			}
			Self::Ref(id, r) => r.process(vocabulary, generator, stack, triples, context, id),
			Self::RefUnion(id, r) => r.process(vocabulary, generator, stack, triples, context, id),
			Self::Object(id, o) => o.process(vocabulary, generator, stack, triples, context, id),
			Self::ObjectProperty(id, p) => {
				p.process(vocabulary, generator, stack, triples, context, id)
			}
			Self::Boolean(id, b) => b.process(vocabulary, generator, stack, triples, context, id),
			Self::Integer(id, i) => i.process(vocabulary, generator, stack, triples, context, id),
			Self::String(id, s) => s.process(vocabulary, generator, stack, triples, context, id),
			Self::Blob(id, b) => b.process(vocabulary, generator, stack, triples, context, id),
			Self::Ipld(id, i) => i.process(vocabulary, generator, stack, triples, context, id),
			Self::Unknown(id, u) => u.process(vocabulary, generator, stack, triples, context, id),
		}
	}
}

impl<V: VocabularyMut> Process<V> for LexAnyUserType {
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
			Self::Record(r) => r.process(vocabulary, generator, stack, triples, context, id),
			Self::Query(q) => q.process(vocabulary, generator, stack, triples, context, id),
			Self::Procedure(p) => p.process(vocabulary, generator, stack, triples, context, id),
			Self::Subscription(s) => s.process(vocabulary, generator, stack, triples, context, id),
			Self::Array(a) => a.process(vocabulary, generator, stack, triples, context, id),
			Self::Token(t) => t.process(vocabulary, generator, stack, triples, context, id),
			Self::Object(o) => o.process(vocabulary, generator, stack, triples, context, id),
			Self::Boolean(b) => b.process(vocabulary, generator, stack, triples, context, id),
			Self::Integer(i) => i.process(vocabulary, generator, stack, triples, context, id),
			Self::String(s) => s.process(vocabulary, generator, stack, triples, context, id),
			Self::Bytes(b) => b.process(vocabulary, generator, stack, triples, context, id),
			Self::CidLink(l) => l.process(vocabulary, generator, stack, triples, context, id),
			Self::Unknown(u) => stack.push(Item::Unknown(id, u)),
		}
	}
}

fn nsid_name(s: &str) -> &str {
	match s.rsplit_once('/') {
		Some((_, r)) => r,
		None => s,
	}
}

fn sub_id<V: VocabularyMut>(
	vocabulary: &mut V,
	id: &OutputSubject<V>,
	name: &str,
) -> OutputSubject<V> {
	let iri = IriBuf::new(&format!(
		"{}/{}",
		vocabulary.iri(id.as_iri().unwrap()).unwrap(),
		name
	))
	.unwrap();

	Id::Iri(vocabulary.insert(iri.as_iri()))
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
