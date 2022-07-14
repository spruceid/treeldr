/// Identifier type.
pub trait Id: 'static {
	type Ref<'a>: PartialEq<Self>;

	fn as_ref(&self) -> Self::Ref<'_>;

	fn from_ref(r: Self::Ref<'_>) -> Self;
}

impl Id for rdf_types::Subject {
	type Ref<'a> = rdf_types::SubjectRef<'a>;

	fn as_ref(&self) -> Self::Ref<'_> {
		self.as_subject_ref()
	}

	fn from_ref(r: Self::Ref<'_>) -> Self {
		match r {
			rdf_types::SubjectRef::Blank(b) => Self::Blank(b.to_owned()),
			rdf_types::SubjectRef::Iri(i) => Self::Iri(i.into())
		}
	}
}

pub mod xsd {
	use rdf_types::SubjectRef;
	use static_iref::iri;

	macro_rules! id {
		{ $($tr:ident : $c:ident = $iri:expr),* } => {
			$(
				pub trait $tr: crate::Id {
					const $c: Self::Ref<'static>;
				}

				impl $tr for rdf_types::Subject {
					const $c: SubjectRef<'static> = $iri;
				}
			)*
		};
	}

	id! {
		Boolean : XSD_BOOLEAN = SubjectRef::Iri(iri!("http://www.w3.org/2001/XMLSchema#boolean")),
		String : XSD_STRING = SubjectRef::Iri(iri!("http://www.w3.org/2001/XMLSchema#string")),
		Integer : XSD_INTEGER = SubjectRef::Iri(iri!("http://www.w3.org/2001/XMLSchema#integer")),
		DateTime : XSD_DATE_TIME = SubjectRef::Iri(iri!("http://www.w3.org/2001/XMLSchema#dateTime"))
	}
}