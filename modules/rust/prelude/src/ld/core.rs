use std::hash::Hash;

use json_ld::{object::LiteralString, syntax::Entry};
use locspan::Meta;
use rdf_types::{IntoId, Namespace, Subject, VocabularyMut};
use static_iref::iri;

use crate::Id;

/// JSON-LD document serialization.
pub trait IntoJsonLd<V: VocabularyMut, M = ()> {
	/// Converts the value into a JSON-LD document.
	fn into_json_ld(
		self,
		vocabulary: &mut V,
	) -> Meta<json_ld::ExpandedDocument<V::Iri, V::BlankId, M>, M>;
}

/// JSON-LD object serialization.
pub trait IntoJsonLdObject<V: VocabularyMut, M = ()> {
	/// Converts the value into a JSON-LD object.
	fn into_json_ld_object(
		self,
		vocabulary: &mut V,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M>;
}

/// JSON-LD object serialization with metadata.
///
/// The [`IntoJsonLdObject`] trait should be used instead, which is implemented
/// for `Meta<T, M>` where `T: IntoJsonLdObjectMeta`.
pub trait IntoJsonLdObjectMeta<V: VocabularyMut, M = ()> {
	/// Converts the value into a JSON-LD object with the given metadata.
	fn into_json_ld_object_meta(
		self,
		vocabulary: &mut V,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M>;
}

impl<V: VocabularyMut, M, T: IntoJsonLdObjectMeta<V, M>> IntoJsonLdObject<V, M> for Meta<T, M> {
	fn into_json_ld_object(
		self,
		vocabulary: &mut V,
	) -> json_ld::IndexedObject<<V>::Iri, <V>::BlankId, M> {
		self.0.into_json_ld_object_meta(vocabulary, self.1)
	}
}

impl<V: VocabularyMut, T: IntoJsonLdObject<V, M>, M> IntoJsonLd<V, M> for T
where
	V::Iri: Eq + Hash,
	V::BlankId: Eq + Hash,
	M: Clone,
{
	fn into_json_ld(
		self,
		vocabulary: &mut V,
	) -> Meta<json_ld::ExpandedDocument<V::Iri, V::BlankId, M>, M> {
		let object = self.into_json_ld_object(vocabulary);
		let mut result = json_ld::ExpandedDocument::new();
		let meta = object.metadata().clone();
		result.insert(object);
		Meta(result, meta)
	}
}

impl<V: VocabularyMut, M> IntoJsonLdObjectMeta<V, M> for bool {
	fn into_json_ld_object_meta(
		self,
		vocabulary: &mut V,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Value(json_ld::Value::Literal(
					json_ld::object::Literal::Boolean(self),
					Some(vocabulary.insert(iri!("http://www.w3.org/2001/XMLSchema#boolean"))),
				)),
				None,
			),
			meta,
		)
	}
}

impl<V: VocabularyMut, M> IntoJsonLdObjectMeta<V, M> for String {
	fn into_json_ld_object_meta(
		self,
		_vocabulary: &mut V,
		meta: M,
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, M> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Value(json_ld::Value::Literal(
					json_ld::object::Literal::String(LiteralString::Inferred(self)),
					None,
				)),
				None,
			),
			meta,
		)
	}
}

impl<V: VocabularyMut, M> IntoJsonLdObjectMeta<V, M> for chrono::NaiveDate {
	fn into_json_ld_object_meta(
		self,
		vocabulary: &mut V,
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

impl<V: VocabularyMut, M> IntoJsonLdObjectMeta<V, M> for chrono::DateTime<chrono::Utc> {
	fn into_json_ld_object_meta(
		self,
		vocabulary: &mut V,
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

impl<V: VocabularyMut, M> IntoJsonLdObjectMeta<V, M> for iref::IriBuf {
	fn into_json_ld_object_meta(
		self,
		vocabulary: &mut V,
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

impl<V: VocabularyMut, M> IntoJsonLdObjectMeta<V, M> for iref::IriRefBuf {
	fn into_json_ld_object_meta(
		self,
		vocabulary: &mut V,
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

impl<V: VocabularyMut> IntoJsonLdObjectMeta<V> for Subject<V::Iri, V::BlankId> {
	fn into_json_ld_object_meta(
		self,
		_vocabulary: &mut V,
		meta: (),
	) -> json_ld::IndexedObject<V::Iri, V::BlankId, ()> {
		Meta(
			json_ld::Indexed::new(
				json_ld::Object::Node(Box::new(json_ld::Node::with_id(Entry::new(
					(),
					Meta(self.into(), ()),
				)))),
				None,
			),
			meta,
		)
	}
}

impl<N: VocabularyMut> IntoJsonLdObjectMeta<N> for Id<N::Id>
where
	N: Namespace,
	N::Id: IntoId<Iri = N::Iri, BlankId = N::BlankId>,
{
	fn into_json_ld_object_meta(
		self,
		vocabulary: &mut N,
		meta: (),
	) -> json_ld::IndexedObject<N::Iri, N::BlankId, ()> {
		self.0.into_id().into_json_ld_object_meta(vocabulary, meta)
	}
}
