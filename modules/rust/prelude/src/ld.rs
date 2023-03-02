use rdf_types::Subject;

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