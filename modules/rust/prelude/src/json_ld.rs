use crate::rdf::{Literal, Object, Subject};
use iref::IriBuf;
use json_ld::ValidReference;
use rdf_types::{BlankIdBuf, Quad};

pub trait IntoJsonLd<M> {
	fn into_json_ld(self) -> json_ld::syntax::Value<M>;
}

impl<M> IntoJsonLd<M> for bool {
	fn into_json_ld(self) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::Boolean(self)
	}
}

impl<M> IntoJsonLd<M> for String {
	fn into_json_ld(self) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::String(self.into())
	}
}

impl<M> IntoJsonLd<M> for chrono::DateTime<chrono::Utc> {
	fn into_json_ld(self) -> json_ld::syntax::Value<M> {
		json_ld::syntax::Value::String(self.to_string().into())
	}
}

impl<I: std::fmt::Display, B: std::fmt::Display, M> IntoJsonLd<M> for Subject<I, B> {
	fn into_json_ld(self) -> json_ld::syntax::Value<M> {
		match self {
			Subject::Iri(i) => json_ld::syntax::Value::String(i.to_string().into()),
			Subject::Blank(b) => json_ld::syntax::Value::String(b.to_string().into()),
		}
	}
}

pub type ImportedQuad<'a> =
	Quad<Subject<IriBuf>, Subject<IriBuf>, Object<Subject<IriBuf>>, &'a ValidReference<IriBuf>>;

pub fn import_quad(
	Quad(subject, predicate, object, graph): json_ld::rdf::QuadRef<IriBuf, BlankIdBuf>,
) -> ImportedQuad {
	let subject = subject.as_ref().clone().into_rdf_subject();
	let predicate = predicate.as_ref().clone().into_rdf_subject();

	let object = match object {
		json_ld::rdf::Value::Literal(l) => Object::Literal(import_literal(l)),
		json_ld::rdf::Value::Reference(r) => Object::Id(r.into_rdf_subject()),
	};

	Quad(subject, predicate, object, graph)
}

pub fn import_literal(lit: json_ld::rdf::Literal<IriBuf>) -> Literal<Subject<IriBuf>> {
	match lit {
		json_ld::rdf::Literal::String(s) => Literal::String(s),
		json_ld::rdf::Literal::TypedString(s, ty) => Literal::TypedString(s, Subject::Iri(ty)),
		json_ld::rdf::Literal::LangString(s, l) => Literal::LangString(s, l),
	}
}
