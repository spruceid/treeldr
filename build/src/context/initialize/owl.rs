use treeldr::{metadata::Merge, vocab, Id, IriIndex};

use crate::Context;

impl<M> Context<M> {
	pub fn define_owl_types(&mut self, metadata: M)
	where
		M: Clone + Merge,
	{
		use vocab::{Owl, Term};

		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Owl(Owl::Restriction))),
			metadata.clone(),
		);

		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Owl(Owl::FunctionalProperty))),
			metadata,
		);
	}
}
