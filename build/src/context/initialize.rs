use ::treeldr::{metadata::Merge, BlankIdIndex, IriIndex};
use rdf_types::{Generator, VocabularyMut};

use super::Context;

mod owl;
mod rdf;
mod rdfs;
mod treeldr;
mod xsd;

impl<M> Context<M> {
	pub fn apply_built_in_definitions_with<
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		metadata: M,
	) where
		M: Clone + Merge,
	{
		self.define_rdfs_types(vocabulary, generator, metadata.clone());
		self.define_rdf_types(metadata.clone());
		self.define_xsd_types(metadata.clone());
		self.define_owl_types(metadata.clone());
		self.define_treeldr_types(metadata)
	}

	pub fn apply_built_in_definitions<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) where
		M: Default + Clone + Merge,
	{
		self.apply_built_in_definitions_with(vocabulary, generator, M::default())
	}
}
