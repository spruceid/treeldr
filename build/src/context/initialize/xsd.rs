use treeldr::{vocab, IriIndex, metadata::Merge, Id};

use crate::Context;

impl<M> Context<M> {
	pub fn define_xsd_types(&mut self, metadata: M)
	where
		M: Clone + Merge,
	{
		use vocab::{Term, Xsd};
		self.define_primitive_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::String))),
			metadata,
		);
	}
}