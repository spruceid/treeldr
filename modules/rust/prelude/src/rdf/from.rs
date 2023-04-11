use rdf_types::Namespace;
pub use rdf_types::{Literal, Object, Subject};

mod literal;

pub use literal::*;

#[derive(Clone, Copy, Debug)]
pub enum FromRdfError {
	Never,
	UnexpectedLiteralValue,
	ExpectedLiteralValue,
	UnexpectedType,
	InvalidLexicalRepresentation,
	MissingRequiredPropertyValue,
}

/// Import from an RDF graph.
pub trait FromRdf<N: Namespace, L>: Sized {
	fn from_rdf<G>(
		namespace: &mut N,
		value: &Object<N::Id, L>,
		graph: &G,
	) -> Result<Self, FromRdfError>
	where
		G: grdf::Graph<Subject = N::Id, Predicate = N::Id, Object = Object<N::Id, L>>;
}

impl<N: Namespace, L> FromRdf<N, L> for crate::Id<N::Id>
where
	N::Id: Clone,
{
	fn from_rdf<G>(
		_namespace: &mut N,
		value: &Object<<N as Namespace>::Id, L>,
		_graph: &G,
	) -> Result<Self, FromRdfError>
	where
		G: grdf::Graph<
			Subject = <N as Namespace>::Id,
			Predicate = <N as Namespace>::Id,
			Object = Object<<N as Namespace>::Id, L>,
		>,
	{
		match value {
			Object::Id(id) => Ok(Self(id.clone())),
			_ => Err(FromRdfError::UnexpectedLiteralValue),
		}
	}
}

macro_rules! from_rdf_literal {
	($($ty:ty),*) => {
		$(
			impl<N: Namespace, L> FromRdf<N, L> for $ty where $ty: FromLiteral<L, N> {
				fn from_rdf<G>(namespace: &mut N, value: &Object<<N as Namespace>::Id, L>, _graph: &G) -> Result<Self, FromRdfError>
					where
						G: grdf::Graph<Subject = <N as Namespace>::Id, Predicate = <N as Namespace>::Id, Object = Object<<N as Namespace>::Id, L>> {
					match value {
						Object::Literal(l) => Self::from_literal(namespace, l),
						Object::Id(_) => Err(FromRdfError::ExpectedLiteralValue)
					}
				}
			}
		)*
	};
}

from_rdf_literal! {
	bool,
	i64,
	String,
	iref::IriBuf,
	::chrono::NaiveDate,
	::chrono::DateTime<::chrono::Utc>
}

/// Literal value type check.
pub trait TypeCheck<T> {
	fn has_type(&self, ty: &T) -> bool;
}

impl<S, I: RdfType, L> TypeCheck<I> for Literal<S, I, L> {
	fn has_type(&self, ty: &I) -> bool {
		match self {
			Literal::String(_) => ty.is_string(),
			Literal::LangString(_, _) => ty.is_lang_string(),
			Literal::TypedString(_, t) => t == ty,
		}
	}
}

impl<S, I: RdfType, B, L> TypeCheck<rdf_types::Id<I, B>> for Literal<S, I, L> {
	fn has_type(&self, ty: &rdf_types::Id<I, B>) -> bool {
		match ty {
			rdf_types::Id::Iri(ty) => match self {
				Literal::String(_) => ty.is_string(),
				Literal::LangString(_, _) => ty.is_lang_string(),
				Literal::TypedString(_, t) => t == ty,
			},
			rdf_types::Id::Blank(_) => false,
		}
	}
}
pub trait RdfType: PartialEq {
	fn is_string(&self) -> bool;

	fn is_lang_string(&self) -> bool;
}

impl<'a> RdfType for iref::Iri<'a> {
	fn is_string(&self) -> bool {
		*self == static_iref::iri!("http://www.w3.org/2001/XMLSchema#string")
	}

	fn is_lang_string(&self) -> bool {
		*self == static_iref::iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString")
	}
}

impl RdfType for iref::IriBuf {
	fn is_string(&self) -> bool {
		*self == static_iref::iri!("http://www.w3.org/2001/XMLSchema#string")
	}

	fn is_lang_string(&self) -> bool {
		*self == static_iref::iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString")
	}
}
