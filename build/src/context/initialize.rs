use std::fmt;

use iref::IriBuf;
use treeldr::vocab::IndexedVocabulary;
use ::treeldr::{
	metadata::Merge,
	vocab::{GraphLabel, Object},
	Id
};
use locspan::{Meta, Span};
use nquads_syntax::Parse;
use rdf_types::{generator::Unscoped, vocabulary::Scoped, Generator, VocabularyMut, InsertIntoVocabulary, Quad};

use crate::Document;

use super::Context;

#[derive(Debug, Clone, Copy)]
pub enum Scope {
	Rdf,
	Rdfs,
	RdfsTldr,
	Xsd,
	XsdTldr,
	Owl,
	Tldr,
}

impl Scope {
	pub fn name(&self) -> &'static str {
		match self {
			Scope::Rdf => "rdf",
			Scope::Rdfs => "rdfs",
			Scope::RdfsTldr => "rdfs-tldr",
			Scope::Xsd => "xsd",
			Scope::XsdTldr => "xsd-tldr",
			Scope::Owl => "owl",
			Scope::Tldr => "tldr",
		}
	}
}

impl fmt::Display for Scope {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.name().fmt(f)
	}
}

type RdfTerm<M> = rdf_types::Term<rdf_types::Id, rdf_types::Literal<Meta<rdf_types::literal::Type, M>, Meta<String, M>>>;
type RdfQuad<M> = Quad<Meta<rdf_types::Id, M>, Meta<IriBuf, M>, Meta<RdfTerm<M>, M>, Meta<rdf_types::Id, M>>;

fn import_quad<M>(
	vocabulary: &mut (impl IndexedVocabulary + VocabularyMut),
	Meta(Quad(s, p, o, g), meta): Meta<RdfQuad<M>, M>
) -> Meta<Quad<Meta<Id, M>, Meta<Id, M>, Meta<Object<M>, M>, Meta<GraphLabel, M>>, M> {
	Meta(Quad(
		s.insert_into_vocabulary(vocabulary),
		p.insert_into_vocabulary(vocabulary).map(Id::Iri),
		match o {
			Meta(rdf_types::Term::Id(id), m) => Meta(Object::Id(id.insert_into_vocabulary(vocabulary)), m),
			Meta(rdf_types::Term::Literal(l), m) => Meta(Object::Literal(l.insert_type_into_vocabulary(vocabulary)), m)
		},
		g.insert_into_vocabulary(vocabulary)
	), meta)
}

impl<M> Context<M> {
	pub fn import_nquads<V: IndexedVocabulary + VocabularyMut>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		scope: Scope,
		content: &str,
		metadata: impl FnMut(Span) -> M,
	) where
		M: Clone + Ord + Merge,
	{
		let mut scoped_vocabulary = Scoped::new(vocabulary, scope);
		let mut unscoped_generator = Unscoped(generator);

		let doc = nquads_syntax::Document::parse_str(content, metadata)
			.ok()
			.unwrap();
		let dataset: grdf::meta::BTreeDataset<Id, Id, Object<M>, GraphLabel, M> = doc
			.into_value()
			.into_iter()
			.map(|quad| import_quad(&mut scoped_vocabulary, quad))
			.collect();

		dataset
			.declare(
				&mut (),
				self,
				&mut scoped_vocabulary,
				&mut unscoped_generator,
			)
			.ok()
			.unwrap();
		dataset
			.define(
				&mut (),
				self,
				&mut scoped_vocabulary,
				&mut unscoped_generator,
			)
			.ok()
			.unwrap();
	}

	pub fn apply_built_in_definitions_with<
		V: IndexedVocabulary + VocabularyMut,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		metadata: M,
	) where
		M: Clone + Ord + Merge,
	{
		self.import_nquads(
			vocabulary,
			generator,
			Scope::Rdf,
			include_str!("../../../schema/rdf.nq"),
			|_| metadata.clone(),
		);

		self.import_nquads(
			vocabulary,
			generator,
			Scope::Rdfs,
			include_str!("../../../schema/rdfs.nq"),
			|_| metadata.clone(),
		);

		self.import_nquads(
			vocabulary,
			generator,
			Scope::RdfsTldr,
			include_str!("../../../schema/rdfs+tldr.nq"),
			|_| metadata.clone(),
		);

		self.import_nquads(
			vocabulary,
			generator,
			Scope::Xsd,
			include_str!("../../../schema/xsd.nq"),
			|_| metadata.clone(),
		);

		self.import_nquads(
			vocabulary,
			generator,
			Scope::XsdTldr,
			include_str!("../../../schema/xsd+tldr.nq"),
			|_| metadata.clone(),
		);

		self.import_nquads(
			vocabulary,
			generator,
			Scope::Owl,
			include_str!("../../../schema/owl.nq"),
			|_| metadata.clone(),
		);

		self.import_nquads(
			vocabulary,
			generator,
			Scope::Tldr,
			include_str!("../../../schema/tldr.nq"),
			|_| metadata.clone(),
		);
	}

	pub fn apply_built_in_definitions<V: IndexedVocabulary + VocabularyMut>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) where
		M: Default + Ord + Clone + Merge,
	{
		self.apply_built_in_definitions_with(vocabulary, generator, M::default())
	}
}
