use locspan::Meta;
use treeldr::{metadata::Merge, vocab, Id, IriIndex};

use crate::Context;

impl<M> Context<M> {
	pub fn define_rdf_types(&mut self, metadata: M)
	where
		M: Clone + Merge,
	{
		use vocab::{Rdf, Rdfs, Term};

		// rdf:Property
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Property))),
			metadata.clone(),
		);

		// rdf:type
		let prop = self
			.declare_property(
				Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Type))),
				metadata.clone(),
			)
			.as_property_mut();
		prop.domain_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		));
		prop.range_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Class))),
			metadata.clone(),
		));

		// rdf:List
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
			metadata.clone(),
		);

		// rdf:first
		let prop = self
			.declare_property(
				Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
				metadata.clone(),
			)
			.as_property_mut();
		prop.domain_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
			metadata.clone(),
		));
		prop.range_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		));

		// rdf:rest
		let prop = self
			.declare_property(
				Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
				metadata.clone(),
			)
			.as_property_mut();
		prop.domain_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
			metadata.clone(),
		));
		prop.range_mut()
			.insert(Meta(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))), metadata))
	}
}
