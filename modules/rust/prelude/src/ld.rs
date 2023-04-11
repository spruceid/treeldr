use std::borrow::Cow;

use contextual::WithContext;
use rdf_types::{Literal, Namespace, Object, Quad, Subject};

use crate::Id;

pub trait IntoJsonLd<N, M = ()> {
	fn into_json_ld(self, namespace: &N) -> json_ld::syntax::Value<M>;
}

impl<N, M> IntoJsonLd<N, M> for bool {
	fn into_json_ld(self, _namespace: &N) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::Boolean(self)
	}
}

impl<N, M> IntoJsonLd<N, M> for String {
	fn into_json_ld(self, _namespace: &N) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::String(self.into())
	}
}

impl<N, M> IntoJsonLd<N, M> for chrono::NaiveDate {
	fn into_json_ld(self, _namespace: &N) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::String(self.to_string().into())
	}
}

impl<N, M> IntoJsonLd<N, M> for chrono::DateTime<chrono::Utc> {
	fn into_json_ld(self, _namespace: &N) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::String(self.to_string().into())
	}
}

impl<N, M> IntoJsonLd<N, M> for iref::IriBuf {
	fn into_json_ld(self, _namespace: &N) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::String(self.to_string().into())
	}
}

impl<N, M> IntoJsonLd<N, M> for iref::IriRefBuf {
	fn into_json_ld(self, _namespace: &N) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::String(self.to_string().into())
	}
}

impl<N, I: std::fmt::Display, B: std::fmt::Display, M> IntoJsonLd<N, M> for Subject<I, B> {
	fn into_json_ld(self, _namespace: &N) -> json_ld::syntax::Value<M> {
		match self {
			Subject::Iri(i) => json_ld::syntax::Value::String(i.to_string().into()),
			Subject::Blank(b) => json_ld::syntax::Value::String(b.to_string().into()),
		}
	}
}

impl<N: Namespace, M> IntoJsonLd<N, M> for Id<N::Id>
where
	N::Id: contextual::DisplayWithContext<N>,
{
	fn into_json_ld(self, namespace: &N) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::String(self.0.with(namespace).to_string().into())
	}
}

/// RDF Quad imported from the `json_ld` library by using the [`import_quad`]
/// function to clone the components of a `GeneratedQuad`.
pub type ImportedQuad<'a, I, B> = Quad<
	rdf_types::Id<I, B>,
	rdf_types::Id<I, B>,
	Object<rdf_types::Id<I, B>, Literal<String, I>>,
	&'a rdf_types::Id<I, B>,
>;

/// RDF Quad generated by the `json_ld` library.
pub type GeneratedQuad<'a, I, B> = rdf_types::Quad<
	Cow<'a, rdf_types::Id<I, B>>,
	Cow<'a, rdf_types::Id<I, B>>,
	Object<rdf_types::Id<I, B>, Literal<String, I>>,
	&'a rdf_types::Id<I, B>,
>;

/// Converts a `GeneratedQuad` into an `ImportedQuad` by copying the borrowed
/// subject and predicate.
pub fn import_quad<I: Clone, B: Clone>(
	Quad(subject, predicate, object, graph): GeneratedQuad<I, B>,
) -> ImportedQuad<I, B> {
	let subject = subject.as_ref().clone();
	let predicate = predicate.as_ref().clone();
	Quad(subject, predicate, object, graph)
}
