use rdf_types::{VocabularyMut, Generator};
use ::treeldr::{IriIndex, BlankIdIndex, metadata::Merge};

use super::Context;

mod rdf;
mod xsd;
mod treeldr;

impl<M> Context<M> {
	pub fn apply_built_in_definitions_with<
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		metadata: M,
	)
	where
		M: Clone + Merge,
	{
		self.define_rdf_types(vocabulary, generator, metadata.clone());
		self.define_xsd_types(metadata.clone());
		self.define_treeldr_types(metadata)
	}

	pub fn apply_built_in_definitions<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	)
	where
		M: Default + Clone + Merge,
	{
		self.apply_built_in_definitions_with(vocabulary, generator, M::default())
	}
}