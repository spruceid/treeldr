use std::fmt;

use ::treeldr::{
	metadata::Merge,
	vocab::{GraphLabel, Object},
	BlankIdIndex, Id, IriIndex,
};
use locspan::{Meta, Span};
use nquads_syntax::Parse;
use rdf_types::{generator::Unscoped, vocabulary::Scoped, Generator, VocabularyMut};

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

impl<M> Context<M> {
	pub fn import_nquads<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
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
			.map(|Meta(quad, meta)| {
				Meta(
					quad.insert_into(&mut scoped_vocabulary)
						.map_predicate(|p| p.map(Id::Iri)),
					meta,
				)
			})
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
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
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

	pub fn apply_built_in_definitions<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) where
		M: Default + Ord + Clone + Merge,
	{
		self.apply_built_in_definitions_with(vocabulary, generator, M::default())
	}
}
