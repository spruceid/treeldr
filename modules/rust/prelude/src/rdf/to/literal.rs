use iref::IriBuf;
use rdf_types::{Generator, IriVocabulary, IriVocabularyMut, Literal, Namespace};
use static_iref::iri;

use crate::rdf::{LiteralValue, ValuesOnly};

use super::QuadsAndValues;

pub trait AsLiteral<N, L> {
	fn rdf_literal_value(&self, namespace: &mut N) -> L;
}

macro_rules! impl_as_literal {
	{ $($ty:ty : $rdf_ty:tt),* } => {
		$(
			impl<N: IriVocabularyMut, L> AsLiteral<N, L> for $ty
			where
				L: From<Literal<String, N::Iri>>,
			{
				fn rdf_literal_value(&self, namespace: &mut N) -> L {
					Literal::TypedString(
						self.to_string(),
						namespace.insert(iri!($rdf_ty))
					).into()
				}
			}

			impl<N: Namespace + IriVocabulary, L> QuadsAndValues<N, L> for $ty
			where
				Self: AsLiteral<N, L>,
			{
				type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

				fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
					&'a self,
					_namespace: &mut N,
					_generator: &mut G,
				) -> Self::QuadsAndValues<'a>
				where
					N::Id: 'a,
					L: 'a,
				{
					ValuesOnly::new(LiteralValue::new(self))
				}
			}
		)*
	};
}

impl_as_literal! {
	xsd_types::Boolean: "http://www.w3.org/2001/XMLSchema#boolean",
	xsd_types::Integer: "http://www.w3.org/2001/XMLSchema#integer",
	xsd_types::Long: "http://www.w3.org/2001/XMLSchema#long",
	xsd_types::Int: "http://www.w3.org/2001/XMLSchema#int",
	xsd_types::Short: "http://www.w3.org/2001/XMLSchema#short",
	xsd_types::Byte: "http://www.w3.org/2001/XMLSchema#byte",
	xsd_types::NonNegativeInteger: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
	xsd_types::PositiveInteger: "http://www.w3.org/2001/XMLSchema#positiveInteger",
	xsd_types::UnsignedLong: "http://www.w3.org/2001/XMLSchema#unsignedLong",
	xsd_types::UnsignedInt: "http://www.w3.org/2001/XMLSchema#unsignedInt",
	xsd_types::UnsignedShort: "http://www.w3.org/2001/XMLSchema#unsignedShort",
	xsd_types::UnsignedByte: "http://www.w3.org/2001/XMLSchema#unsignedByte",
	xsd_types::NonPositiveInteger: "http://www.w3.org/2001/XMLSchema#nonPositiveInteger",
	xsd_types::NegativeInteger: "http://www.w3.org/2001/XMLSchema#negativeInteger",
	xsd_types::Base64BinaryBuf: "http://www.w3.org/2001/XMLSchema#base64Binary",
	xsd_types::HexBinaryBuf: "http://www.w3.org/2001/XMLSchema#hexBinary"
}

impl<N: IriVocabulary, L> AsLiteral<N, L> for String
where
	L: From<Literal<String, N::Iri>>,
{
	fn rdf_literal_value(&self, _namespace: &mut N) -> L {
		Literal::String(self.to_owned()).into()
	}
}

impl<N: Namespace + IriVocabulary, L> QuadsAndValues<N, L> for String
where
	Self: AsLiteral<N, L>,
{
	type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::QuadsAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(LiteralValue::new(self))
	}
}

impl<N: Namespace + IriVocabulary, L> QuadsAndValues<N, L> for IriBuf
where
	Self: AsLiteral<N, L>,
{
	type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::QuadsAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(LiteralValue::new(self))
	}
}

impl<N: Namespace + IriVocabulary, L> QuadsAndValues<N, L> for chrono::NaiveDate
where
	Self: AsLiteral<N, L>,
{
	type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::QuadsAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(LiteralValue::new(self))
	}
}

impl<N: Namespace + IriVocabulary, L> QuadsAndValues<N, L> for chrono::DateTime<chrono::Utc>
where
	Self: AsLiteral<N, L>,
{
	type QuadsAndValues<'a> = ValuesOnly<LiteralValue<'a, Self, N::Id, L>> where Self: 'a, N::Id: 'a, L: 'a;

	fn unbound_rdf_quads_and_values<'a, G: Generator<N>>(
		&'a self,
		_namespace: &mut N,
		_generator: &mut G,
	) -> Self::QuadsAndValues<'a>
	where
		N::Id: 'a,
		L: 'a,
	{
		ValuesOnly::new(LiteralValue::new(self))
	}
}
