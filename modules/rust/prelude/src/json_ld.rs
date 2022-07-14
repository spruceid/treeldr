use rdf_types::Quad;
use json_ld::{ValidReference};
use crate::{Subject, Object, Literal};
use iref::IriBuf;

pub trait IntoJsonLd<C, M> {
	fn into_json_ld(self) -> json_ld::syntax::Value<C, M>;
}

impl<C, M> IntoJsonLd<C, M> for bool {
	fn into_json_ld(self) -> json_ld::syntax::Value<C, M> {
		json_ld::syntax::Value::Boolean(self)
	}
}

impl<C, M> IntoJsonLd<C, M> for String {
	fn into_json_ld(self) -> json_ld::syntax::Value<C, M> {
		json_ld::syntax::Value::String(self.into())
	}
}

impl<C, M> IntoJsonLd<C, M> for chrono::DateTime<chrono::Utc> {
	fn into_json_ld(self) -> json_ld::syntax::Value<C, M> {
		json_ld::syntax::Value::String(self.to_string().into())
	}
}

impl<I: std::fmt::Display, B: std::fmt::Display, C, M> IntoJsonLd<C, M> for crate::Subject<I, B> {
	fn into_json_ld(self) -> json_ld::syntax::Value<C, M> {
		match self {
			crate::Subject::Iri(i) => json_ld::syntax::Value::String(i.to_string().into()),
			crate::Subject::Blank(b) => json_ld::syntax::Value::String(b.to_string().into())
		}
	}
}

pub fn import_quad<'a>(
	Quad(subject, predicate, object, graph): json_ld::rdf::QuadRef<'a, IriBuf>
) -> Quad<Subject<IriBuf>, Subject<IriBuf>, Object<Subject<IriBuf>>, &'a ValidReference<IriBuf>> {
	let subject = subject.as_ref().clone().into_rdf_subject();

	let predicate = match predicate {
		json_ld::rdf::PropertyRef::Other(r) => r.clone().into_rdf_subject(),
		json_ld::rdf::PropertyRef::Rdf(p) => Subject::Iri(p.as_iri().into())
	};

	let object = match object {
		json_ld::rdf::Value::Literal(l) => Object::Literal(import_literal(l)),
		json_ld::rdf::Value::Reference(r) => Object::Id(r.into_rdf_subject()),
		json_ld::rdf::Value::Nil => Object::Id(Subject::Iri(json_ld::rdf::NIL_IRI.into()))
	};

	Quad(subject, predicate, object, graph)
}

pub fn import_literal(lit: json_ld::rdf::Literal<IriBuf>) -> Literal<Subject<IriBuf>> {
	match lit {
		json_ld::rdf::Literal::String(s) => Literal::String(s),
		json_ld::rdf::Literal::TypedString(s, ty) => Literal::TypedString(s, import_literal_type(ty)),
		json_ld::rdf::Literal::LangString(s, l) => Literal::LangString(s, l)
	}
}

pub fn import_literal_type(ty: json_ld::rdf::LiteralType<IriBuf>) -> Subject<IriBuf> {
	match ty {
		json_ld::rdf::LiteralType::Rdfs(ty) => Subject::Iri(ty.as_iri().into()),
		json_ld::rdf::LiteralType::Xsd(ty) => Subject::Iri(ty.as_iri().into()),
		json_ld::rdf::LiteralType::I18n(ty) => Subject::Iri(ty.as_iri().into()),
		json_ld::rdf::LiteralType::Other(iri) => Subject::Iri(iri)
	}
}