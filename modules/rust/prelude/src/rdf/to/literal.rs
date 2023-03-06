use iref::IriBuf;
use rdf_types::{Generator, IriVocabulary, Literal, Namespace};

use crate::rdf::{LiteralValue, ValuesOnly};

use super::TriplesAndValues;

pub trait AsLiteral<N, L> {
	fn rdf_literal_value(&self, namespace: &mut N) -> L;
}

impl<N: IriVocabulary, L> AsLiteral<N, L> for String
where
	L: From<Literal<String, N::Iri>>,
{
	fn rdf_literal_value(&self, _namespace: &mut N) -> L {
		Literal::String(self.to_owned()).into()
	}
}

impl<N: Namespace + IriVocabulary, L> TriplesAndValues<N, L> for String
where
	Self: AsLiteral<N, L>,
{
	type TriplesAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(LiteralValue::new(self))
	}
}

impl<N: Namespace + IriVocabulary, L> TriplesAndValues<N, L> for IriBuf
where
	Self: AsLiteral<N, L>,
{
	type TriplesAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(LiteralValue::new(self))
	}
}

impl<N: Namespace + IriVocabulary, L> TriplesAndValues<N, L> for chrono::NaiveDate
where
	Self: AsLiteral<N, L>,
{
	type TriplesAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(LiteralValue::new(self))
	}
}

impl<N: Namespace + IriVocabulary, L> TriplesAndValues<N, L> for chrono::DateTime<chrono::Utc>
where
	Self: AsLiteral<N, L>,
{
	type TriplesAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_triples_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(LiteralValue::new(self))
	}
}
