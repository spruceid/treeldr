use locspan::Meta;
use rdf_types::{VocabularyMut, Generator};
use treeldr::{vocab, IriIndex, BlankIdIndex, metadata::Merge, Id};

use crate::Context;

impl<M> Context<M> {
	pub fn define_rdf_types<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		metadata: M,
	)
	where
		M: Clone + Merge,
	{
		use vocab::{Rdf, Rdfs, Term};
		// rdfs:Resource
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		);
		self.declare_layout(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		);
		let id_field = generator.next(vocabulary);
		self.declare_layout_field(id_field, metadata.clone());
		let resource_ref_layout = self.standard_reference(
			vocabulary,
			generator,
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
			metadata.clone(),
		);
		let field_layout = self.create_option_layout(
			vocabulary,
			generator,
			resource_ref_layout,
			metadata.clone(),
		);
		let field = self
			.get_mut(id_field)
			.unwrap()
			.as_layout_field_mut()
			.unwrap();
		field.property_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::Self_))),
			metadata.clone(),
		));
		field.name_mut().insert(Meta(treeldr::Name::new("id").unwrap(), metadata.clone()));
		field.layout_mut().insert(Meta(field_layout, metadata.clone()));
		let fields_id = self.create_list(
			vocabulary,
			generator,
			[Meta(id_field.into_term(), metadata.clone())],
		);
		let layout = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))))
			.unwrap()
			.as_layout_mut()
			.unwrap();
		layout.set_fields(Meta(fields_id, metadata.clone()));

		// rdfs:Class
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Class))),
			metadata.clone(),
		);

		// rdf:Property
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Property))),
			metadata.clone(),
		);

		// rdf:type
		self.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Type))),
			metadata.clone(),
		);
		let prop = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Type))))
			.unwrap()
			.as_property_mut()
			.unwrap();
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
		let list = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))))
			.unwrap()
			.as_type_mut()
			.unwrap();
		list.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
			metadata.clone(),
		);
		list.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
			metadata.clone(),
		);

		// rdf:first
		self.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
			metadata.clone(),
		);
		let prop = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.domain_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
			metadata.clone(),
		));
		prop.range_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
		));

		// rdf:rest
		self.declare_property(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
			metadata.clone(),
		);
		let prop = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))))
			.unwrap()
			.as_property_mut()
			.unwrap();
		prop.domain_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
			metadata.clone(),
		));
		prop.range_mut().insert(Meta(Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))), metadata))
	}
}