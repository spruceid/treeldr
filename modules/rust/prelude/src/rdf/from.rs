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
	fn from_rdf<G>(namespace: &mut N, id: &N::Id, graph: &G) -> Result<Self, FromRdfError>
	where
		G: grdf::Graph<Subject = N::Id, Predicate = N::Id, Object = Object<N::Id, L>>;
}
