use std::hash::Hash;

use json_ld::{object::LiteralString, syntax::Entry};
use locspan::Meta;
use rdf_types::{ReverseTermInterpretation, Subject, VocabularyMut};
use static_iref::iri;

/// JSON-LD document serialization.
pub trait AsJsonLd<V: VocabularyMut, I, M = ()> {
	/// Converts the value into a JSON-LD document.
	fn as_json_ld(
		&self,
		vocabulary: &mut V,
		interpretation: &I,
	) -> Meta<json_ld::ExpandedDocument<V::Iri, V::BlankId, M>, M>;
}

/// JSON-LD object serialization.
pub trait AsJsonLdObject<V: VocabularyMut, I, M = ()> {
	/// Converts the value into a JSON-LD object.
	fn as_json_ld_object(
		&self,
		vocabulary: &mut V,
		interpretation: &I,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M>;
}

/// JSON-LD object serialization with metadata.
///
/// The [`AsJsonLdObject`] trait should be used instead, which is implemented
/// for `Meta<T, M>` where `T: AsJsonLdObjectMeta`.
pub trait AsJsonLdObjectMeta<V: VocabularyMut, I, M = ()> {
	/// Converts the value into a JSON-LD object with the given metadata.
	fn as_json_ld_object_meta(
		&self,
		vocabulary: &mut V,
		interpretation: &I,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M>;
}

impl<
		V: VocabularyMut,
		I: ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
		M: Clone,
	> AsJsonLdObjectMeta<V, I, M> for crate::Id<I::Resource>
where
	V::Iri: Clone,
	V::BlankId: Clone,
{
	fn as_json_ld_object_meta(
		&self,
		_vocabulary: &mut V,
		interpretation: &I,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		let mut node = json_ld::Node::<V::Iri, V::BlankId, M>::new();

		if let Some(id) = interpretation.ids_of(&self.0).next() {
			node.set_id(Some(json_ld::syntax::Entry::new(
				meta.clone(),
				Meta(json_ld::Id::Valid(id.cloned()), meta.clone()),
			)));
		}

		Meta(
			json_ld::Indexed::new(json_ld::Object::Node(Box::new(node)), None),
			meta,
		)
	}
}

impl<V: VocabularyMut, I, M: Clone, T: AsJsonLdObjectMeta<V, I, M>> AsJsonLdObject<V, I, M>
	for Meta<T, M>
{
	fn as_json_ld_object(
		&self,
		vocabulary: &mut V,
		interpretation: &I,
	) -> json_ld::IndexedObject<<V>::Iri, <V>::BlankId, M> {
		self.0
			.as_json_ld_object_meta(vocabulary, interpretation, self.1.clone())
	}
}

impl<V: VocabularyMut, I, T: AsJsonLdObject<V, I, M>, M> AsJsonLd<V, I, M> for T
where
	V::Iri: Eq + Hash,
	V::BlankId: Eq + Hash,
	M: Clone,
{
	fn as_json_ld(
		&self,
		vocabulary: &mut V,
		interpretation: &I,
	) -> Meta<json_ld::ExpandedDocument<V::Iri, V::BlankId, M>, M> {
		let object = self.as_json_ld_object(vocabulary, interpretation);
		let mut result = json_ld::ExpandedDocument::new();
		let meta = object.metadata().clone();
		result.insert(object);
		Meta(result, meta)
	}
}

impl<V: VocabularyMut, I, M> AsJsonLdObjectMeta<V, I, M> for bool {
	fn as_json_ld_object_meta(
		&self,
		vocabulary: &mut V,
		_interpretation: &I,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Value(json_ld::Value::Literal(
					json_ld::object::Literal::Boolean(*self),
					Some(vocabulary.insert(iri!("http://www.w3.org/2001/XMLSchema#boolean"))),
				)),
				None,
			),
			meta,
		)
	}
}

macro_rules! impl_as_json_ld_syntax_literal {
	{ $($ty:ty : $rdf_ty:tt),* } => {
		$(
			impl<V: VocabularyMut, I, M> AsJsonLdObjectMeta<V, I, M> for $ty {
				fn as_json_ld_object_meta(
					&self,
					vocabulary: &mut V,
					_interpretation: &I,
					meta: M,
				) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
					Meta(
						json_ld::Indexed::new(
							json_ld::Object::Value(json_ld::Value::Literal(
								json_ld::object::Literal::String(LiteralString::Inferred(self.to_string())),
								Some(vocabulary.insert(iri!($rdf_ty))),
							)),
							None,
						),
						meta,
					)
				}
			}
		)*
	};
}

impl_as_json_ld_syntax_literal! {
	xsd_types::Decimal: "http://www.w3.org/2001/XMLSchema#decimal",
	xsd_types::Integer: "http://www.w3.org/2001/XMLSchema#integer",
	xsd_types::Long: "http://www.w3.org/2001/XMLSchema#long",
	xsd_types::Int: "http://www.w3.org/2001/XMLSchema#int",
	xsd_types::Short: "http://www.w3.org/2001/XMLSchema#short",
	xsd_types::Byte: "http://www.w3.org/2001/XMLSchema#byte",
	xsd_types::NonNegativeInteger: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
	xsd_types::PositiveInteger: "http://www.w3.org/2001/XMLSchema#positiveInteger",
	xsd_types::UnsignedLong: "http://www.w3.org/2001/XMLSchema#unsignedLong",
	xsd_types::UnsignedInt: "http://www.w3.org/2001/XMLSchema#unsignedInt",
	xsd_types::UnsignedShort: "http://www.w3.org/2001/XMLSchema#unsignedShort",
	xsd_types::UnsignedByte: "http://www.w3.org/2001/XMLSchema#unsignedByte",
	xsd_types::NonPositiveInteger: "http://www.w3.org/2001/XMLSchema#nonPositiveInteger",
	xsd_types::NegativeInteger: "http://www.w3.org/2001/XMLSchema#negativeInteger",
	xsd_types::Double: "http://www.w3.org/2001/XMLSchema#double",
	xsd_types::Float: "http://www.w3.org/2001/XMLSchema#float",
	xsd_types::Base64BinaryBuf: "http://www.w3.org/2001/XMLSchema#base64Binary",
	xsd_types::HexBinaryBuf: "http://www.w3.org/2001/XMLSchema#hexBinary"
}

impl<V: VocabularyMut, I, M> AsJsonLdObjectMeta<V, I, M> for String {
	fn as_json_ld_object_meta(
		&self,
		_vocabulary: &mut V,
		_interpretation: &I,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Value(json_ld::Value::Literal(
					json_ld::object::Literal::String(LiteralString::Inferred(self.clone())),
					None,
				)),
				None,
			),
			meta,
		)
	}
}

impl<V: VocabularyMut, I, M> AsJsonLdObjectMeta<V, I, M> for chrono::NaiveDate {
	fn as_json_ld_object_meta(
		&self,
		vocabulary: &mut V,
		_interpretation: &I,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Value(json_ld::Value::Literal(
					json_ld::object::Literal::String(LiteralString::Inferred(self.to_string())),
					Some(vocabulary.insert(iri!("http://www.w3.org/2001/XMLSchema#date"))),
				)),
				None,
			),
			meta,
		)
	}
}

impl<V: VocabularyMut, I, M> AsJsonLdObjectMeta<V, I, M> for chrono::DateTime<chrono::Utc> {
	fn as_json_ld_object_meta(
		&self,
		vocabulary: &mut V,
		_interpretation: &I,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Value(json_ld::Value::Literal(
					json_ld::object::Literal::String(LiteralString::Inferred(self.to_string())),
					Some(vocabulary.insert(iri!("http://www.w3.org/2001/XMLSchema#dateTime"))),
				)),
				None,
			),
			meta,
		)
	}
}

impl<V: VocabularyMut, I, M> AsJsonLdObjectMeta<V, I, M> for iref::IriBuf {
	fn as_json_ld_object_meta(
		&self,
		vocabulary: &mut V,
		_interpretation: &I,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Value(json_ld::Value::Literal(
					json_ld::object::Literal::String(LiteralString::Inferred(self.to_string())),
					Some(vocabulary.insert(iri!("http://www.w3.org/2001/XMLSchema#anyURI"))),
				)),
				None,
			),
			meta,
		)
	}
}

impl<V: VocabularyMut, I, M> AsJsonLdObjectMeta<V, I, M> for iref::IriRefBuf {
	fn as_json_ld_object_meta(
		&self,
		vocabulary: &mut V,
		_interpretation: &I,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Value(json_ld::Value::Literal(
					json_ld::object::Literal::String(LiteralString::Inferred(self.to_string())),
					Some(vocabulary.insert(iri!("http://www.w3.org/2001/XMLSchema#anyURI"))),
				)),
				None,
			),
			meta,
		)
	}
}

impl<V: VocabularyMut, I> AsJsonLdObjectMeta<V, I> for Subject<V::Iri, V::BlankId>
where
	V::Iri: Clone,
	V::BlankId: Clone,
{
	fn as_json_ld_object_meta(
		&self,
		_vocabulary: &mut V,
		_interpretation: &I,
		meta: (),
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, ()> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Node(Box::new(json_ld::Node::with_id(Entry::new(
					(),
					Meta(self.clone().into(), ()),
				)))),
				None,
			),
			meta,
		)
	}
}

// impl<N: VocabularyMut> AsJsonLdObjectMeta<N> for Id<N::Id>
// where
// 	N: Namespace,
// 	N::Id: Clone + IntoId<Iri = N::Iri, BlankId = N::BlankId>,
// {
// 	fn as_json_ld_object_meta(
// 		&self,
// 		vocabulary: &mut N,
// 		meta: (),
// 	) -> json_ld::IndexedObject<N::Iri, N::BlankId, ()> {
// 		self.0
// 			.clone()
// 			.into_id()
// 			.into_json_ld_object_meta(vocabulary, meta)
// 	}
// }
