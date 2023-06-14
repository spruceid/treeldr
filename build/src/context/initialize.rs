use std::fmt;

use ::treeldr::{
	metadata::Merge,
	vocab::{GraphLabel, StrippedObject},
	Id,
};
use iref::IriBuf;
use locspan::{Meta, Span, Strip};
use nquads_syntax::Parse;
use rdf_types::{
	BlankIdBuf, BlankIdVocabularyMut, Generator, InsertIntoVocabulary, IriVocabularyMut,
	LiteralVocabularyMut, Quad,
};
use treeldr::vocab::TldrVocabulary;

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

type RdfTerm<M> = rdf_types::Term<
	rdf_types::Id,
	rdf_types::Literal<Meta<rdf_types::literal::Type, M>, Meta<String, M>>,
>;
type RdfQuad<M> =
	Quad<Meta<rdf_types::Id, M>, Meta<IriBuf, M>, Meta<RdfTerm<M>, M>, Meta<rdf_types::Id, M>>;

fn import_meta_id<M>(
	vocabulary: &mut TldrVocabulary,
	scope: Scope,
	Meta(id, meta): Meta<rdf_types::Id, M>,
) -> Meta<Id, M> {
	Meta(import_id(vocabulary, scope, id), meta)
}

fn import_id(vocabulary: &mut TldrVocabulary, scope: Scope, id: rdf_types::Id) -> Id {
	match id {
		rdf_types::Id::Iri(i) => Id::Iri(vocabulary.insert_owned(i)),
		rdf_types::Id::Blank(b) => {
			let b = BlankIdBuf::new(format!("_:{}:{}", scope, b.suffix())).unwrap();
			Id::Blank(vocabulary.insert_owned_blank_id(b))
		}
	}
}

type ImportedQuad<M> =
	Meta<Quad<Meta<Id, M>, Meta<Id, M>, Meta<StrippedObject, M>, Meta<GraphLabel, M>>, M>;

fn import_quad<M>(
	vocabulary: &mut TldrVocabulary,
	scope: Scope,
	Meta(Quad(s, p, o, g), meta): Meta<RdfQuad<M>, M>,
) -> ImportedQuad<M> {
	Meta(
		Quad(
			import_meta_id(vocabulary, scope, s),
			p.insert_into_vocabulary(vocabulary).map(Id::Iri),
			match o {
				Meta(rdf_types::Term::Id(id), m) => {
					Meta(StrippedObject::Id(import_id(vocabulary, scope, id)), m)
				}
				Meta(rdf_types::Term::Literal(l), m) => {
					let l = l.insert_type_into_vocabulary(vocabulary).strip();
					Meta(
						StrippedObject::Literal(vocabulary.insert_owned_literal(l)),
						m,
					)
				}
			},
			g.map(|g| import_meta_id(vocabulary, scope, g)),
		),
		meta,
	)
}

impl<M> Context<M> {
	pub fn import_nquads(
		&mut self,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
		scope: Scope,
		content: &str,
		metadata: impl FnMut(Span) -> M,
	) where
		M: Clone + Ord + Merge,
	{
		let doc = nquads_syntax::Document::parse_str(content, metadata)
			.ok()
			.unwrap();
		let dataset: grdf::meta::BTreeDataset<Id, Id, StrippedObject, GraphLabel, M> = doc
			.into_value()
			.into_iter()
			.map(|quad| import_quad(vocabulary, scope, quad))
			.collect();

		dataset
			.declare(&mut (), self, vocabulary, generator)
			.ok()
			.unwrap();
		dataset
			.define(&mut (), self, vocabulary, generator)
			.ok()
			.unwrap();
	}

	pub fn apply_built_in_definitions_with(
		&mut self,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
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

	pub fn apply_built_in_definitions(
		&mut self,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
	) where
		M: Default + Ord + Clone + Merge,
	{
		self.apply_built_in_definitions_with(vocabulary, generator, M::default())
	}
}
