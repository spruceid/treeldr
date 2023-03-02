#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Bar {
	pub foo: Option<Foo>,
}
impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
	for Bar
where
	N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
	N::Id: ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
{
	fn from_rdf<G>(
		namespace: &mut N,
		id: &N::Id,
		graph: &G,
	) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
	where
		G: ::treeldr_rust_prelude::grdf::Graph<
			Subject = N::Id,
			Predicate = N::Id,
			Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
		>,
	{
		Ok(Self {
			foo: {
				let mut objects = graph.objects(
					&id,
					&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
						::treeldr_rust_prelude::static_iref::iri!("https://example.com/Bar/foo"),
					)),
				);
				let object = objects.next();
				if objects.next().is_some() {
					panic!("multiples values on functional property")
				}
				match object {
					Some(object) => {
						Some({
							match object {
								::treeldr_rust_prelude::rdf::Object::Id(id) => {
									::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
								}
								_ => return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								),
							}
						})
					}
					None => None,
				}
			},
		})
	}
}
pub struct BarTriplesAndValues<'a, I, V> {
	id_: Option<I>,
	foo: ::treeldr_rust_prelude::rdf::iter::Optional<FooTriplesAndValues<'a, I, V>>,
}
impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::RdfIterator<N>
	for BarTriplesAndValues<'a, N::Id, V>
where
	N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
	N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
{
	type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
	fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
	) -> Option<Self::Item> {
		self.foo
			.next_with(vocabulary, generator)
			.map(|item| match item {
				::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
					treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
				}
				treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
					treeldr_rust_prelude::rdf::TripleOrValue::Triple(::rdf_types::Triple(
						self.id_.clone().unwrap(),
						treeldr_rust_prelude::rdf_types::FromIri::from_iri(vocabulary.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://example.com/Bar/foo"
							),
						)),
						value,
					))
				}
			})
			.or_else(|| {
				self.id_
					.take()
					.map(::treeldr_rust_prelude::rdf_types::Object::Id)
					.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
			})
	}
}
impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
	::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Bar
where
	N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
	N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
{
	type TriplesAndValues < 'a > = BarTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
	fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		V: 'a,
	{
		BarTriplesAndValues {
			id_: Some(generator.next(namespace)),
			foo: self
				.foo
				.unbound_rdf_triples_and_values(namespace, generator),
		}
	}
}
impl ::treeldr_rust_prelude::IntoJsonLd<()> for Bar {
	fn into_json_ld(self) -> ::treeldr_rust_prelude::json_ld::syntax::Value<()> {
		let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
		if let Some(value) = self.foo {
			result.insert(
				::treeldr_rust_prelude::locspan::Meta("foo".into(), ()),
				::treeldr_rust_prelude::locspan::Meta(
					::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value),
					(),
				),
			);
		}
		result.into()
	}
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Foo {
	pub a: Option<String>,
}
impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
	for Foo
where
	N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
	N::Id: ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
{
	fn from_rdf<G>(
		namespace: &mut N,
		id: &N::Id,
		graph: &G,
	) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
	where
		G: ::treeldr_rust_prelude::grdf::Graph<
			Subject = N::Id,
			Predicate = N::Id,
			Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
		>,
	{
		Ok(Self {
			a: {
				let mut objects = graph.objects(
					&id,
					&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
						::treeldr_rust_prelude::static_iref::iri!("https://example.com/Foo/a"),
					)),
				);
				let object = objects.next();
				if objects.next().is_some() {
					panic!("multiples values on functional property")
				}
				match object {
					Some(object) => Some({
						match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < String as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
					}),
					None => None,
				}
			},
		})
	}
}
pub struct FooTriplesAndValues<'a, I, V> {
	id_: Option<I>,
	a: ::treeldr_rust_prelude::rdf::iter::Optional<
		::treeldr_rust_prelude::rdf::iter::ValuesOnly<
			::treeldr_rust_prelude::rdf::iter::LiteralValue<'a, ::std::string::String, I, V>,
		>,
	>,
}
impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::RdfIterator<N>
	for FooTriplesAndValues<'a, N::Id, V>
where
	N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
	N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
{
	type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
	fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
	) -> Option<Self::Item> {
		self.a
			.next_with(vocabulary, generator)
			.map(|item| match item {
				::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
					treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
				}
				treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
					treeldr_rust_prelude::rdf::TripleOrValue::Triple(::rdf_types::Triple(
						self.id_.clone().unwrap(),
						treeldr_rust_prelude::rdf_types::FromIri::from_iri(vocabulary.insert(
							::treeldr_rust_prelude::static_iref::iri!("https://example.com/Foo/a"),
						)),
						value,
					))
				}
			})
			.or_else(|| {
				self.id_
					.take()
					.map(::treeldr_rust_prelude::rdf_types::Object::Id)
					.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
			})
	}
}
impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
	::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Foo
where
	N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
	N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
{
	type TriplesAndValues < 'a > = FooTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
	fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
		&'a self,
		namespace: &mut N,
		generator: &mut G,
	) -> Self::TriplesAndValues<'a>
	where
		N::Id: 'a,
		V: 'a,
	{
		FooTriplesAndValues {
			id_: Some(generator.next(namespace)),
			a: self.a.unbound_rdf_triples_and_values(namespace, generator),
		}
	}
}
impl ::treeldr_rust_prelude::IntoJsonLd<()> for Foo {
	fn into_json_ld(self) -> ::treeldr_rust_prelude::json_ld::syntax::Value<()> {
		let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
		if let Some(value) = self.a {
			result.insert(
				::treeldr_rust_prelude::locspan::Meta("a".into(), ()),
				::treeldr_rust_prelude::locspan::Meta(
					::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value),
					(),
				),
			);
		}
		result.into()
	}
}
pub type String = ::std::string::String;

fn main() {}