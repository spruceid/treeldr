use rdf_types::{IriVocabulary, Literal, Namespace, Generator};

use crate::rdf::iter::{ValuesOnly, LiteralValue};

use super::TriplesAndValues;

pub trait AsLiteral<N, L> {
	fn rdf_literal_value(
		&self,
		namespace: &mut N
	) -> L;
}

impl<N: IriVocabulary, L> AsLiteral<N, L> for String
where
	L: From<Literal<String, N::Iri>>
{
	fn rdf_literal_value(
		&self,
		_namespace: &mut N
	) -> L {
		Literal::String(self.to_owned()).into()
	}
}

impl<N: Namespace + IriVocabulary, L> TriplesAndValues<N, L> for String 
where
	Self: AsLiteral<N, L>
{
	type TriplesAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a;

	fn unbound_rdf_triples_and_values<G: Generator<N>>(
		&self,
		_namespace: &mut N,
		_generator: &mut G
	) -> Self::TriplesAndValues<'_> {
		ValuesOnly::new(LiteralValue::new(self))
	}
}