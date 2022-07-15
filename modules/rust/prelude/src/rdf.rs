use crate::{id, Id};
pub use rdf_types::{Subject, SubjectRef};

#[derive(Clone, Copy, Debug)]
pub enum FromRdfError {
	Never,
	UnexpectedLiteralValue,
	ExpectedLiteralValue,
	UnexpectedLangString,
	UnexpectedType,
	InvalidLexicalRepresentation,
	MissingRequiredPropertyValue,
}

/// RDF literal with custom identifier type.
pub type Literal<T> = rdf_types::Literal<rdf_types::StringLiteral, T>;

/// RDF object with custom identifier type.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Object<T> {
	Id(T),
	Literal(Literal<T>),
}

/// Import from an RDF graph.
pub trait FromRdf<T: Id>: Sized {
	fn from_rdf<G>(id: T::Ref<'_>, graph: &G) -> Result<Self, FromRdfError>
	where
		G: grdf::Graph<Subject = T, Predicate = T, Object = Object<T>>;
}

pub trait FromXsdLiteral<T>: Sized {
	fn from_xsd_literal(literal: &Literal<T>) -> Result<Self, FromRdfError>;
}

macro_rules! get_lexical {
	($literal:ident: $id:ident) => {
		match $literal {
			Literal::String(s) => s.as_str(),
			Literal::TypedString(s, ty) => {
				if T::$id == *ty {
					s.as_str()
				} else {
					return Err(FromRdfError::UnexpectedType);
				}
			}
			Literal::LangString(_, _) => return Err(FromRdfError::UnexpectedLangString),
		}
	};
}

impl<T: id::xsd::Boolean + PartialEq> FromXsdLiteral<T> for bool {
	fn from_xsd_literal(literal: &Literal<T>) -> Result<Self, FromRdfError> {
		let lexical = get_lexical!(literal: XSD_BOOLEAN);
		match lexical {
			"true" => Ok(true),
			"false" => Ok(false),
			_ => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}
}

impl<T: id::xsd::String + PartialEq> FromXsdLiteral<T> for String {
	fn from_xsd_literal(literal: &Literal<T>) -> Result<Self, FromRdfError> {
		let lexical = get_lexical!(literal: XSD_STRING);
		Ok(lexical.to_string())
	}
}

impl<T: id::xsd::Integer + PartialEq> FromXsdLiteral<T> for i64 {
	fn from_xsd_literal(literal: &Literal<T>) -> Result<Self, FromRdfError> {
		let lexical = get_lexical!(literal: XSD_INTEGER);

		match xsd_types::Integer::new(lexical) {
			Ok(i) => Ok(i.as_str().parse().unwrap()),
			Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}
}

impl<T: id::xsd::DateTime + PartialEq> FromXsdLiteral<T> for ::chrono::DateTime<::chrono::Utc> {
	fn from_xsd_literal(literal: &Literal<T>) -> Result<Self, FromRdfError> {
		let lexical = get_lexical!(literal: XSD_DATE_TIME);

		match lexical.parse::<Self>() {
			Ok(d) => Ok(d),
			Err(_) => Err(FromRdfError::InvalidLexicalRepresentation),
		}
	}
}
