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
	::chrono::DateTime<::chrono::Utc>
}

/// Literal value type check.
pub trait TypeCheck<T> {
	fn has_type(&self, ty: &T) -> bool;
}
