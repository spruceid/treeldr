use locspan::Meta;
use treeldr::{
	metadata::Merge,
	vocab::{self, Rdf, Rdfs},
	Id, IriIndex,
};

use crate::Context;

impl<M> Context<M> {
	pub fn define_owl_types(&mut self, metadata: M)
	where
		M: Clone + Merge,
	{
		use vocab::{Owl, Term};

		let restriction = self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Owl(Owl::Restriction))),
			metadata.clone(),
		);
		restriction.as_type_mut().sub_class_of_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Class))).into(),
			metadata.clone(),
		));

		let functional_property = self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Owl(Owl::FunctionalProperty))),
			metadata.clone(),
		);
		functional_property
			.as_type_mut()
			.sub_class_of_mut()
			.insert(Meta(
				Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Property))).into(),
				metadata,
			))
	}
}
