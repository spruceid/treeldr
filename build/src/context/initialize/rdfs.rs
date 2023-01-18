use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, vocab, BlankIdIndex, Id, IriIndex};

use crate::Context;

impl<M> Context<M> {
	pub fn define_rdfs_types<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		metadata: M,
	) where
		M: Clone + Merge,
	{
		use vocab::{Rdfs, Term};
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
		let resource_ref_layout = self.create_reference(
			vocabulary,
			generator,
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))),
			metadata.clone(),
			metadata.clone(),
		);
		let field_layout =
			self.create_option_layout(vocabulary, generator, resource_ref_layout, metadata.clone());
		let field = self.get_mut(id_field).unwrap();
		field.as_layout_field_mut().property_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::TreeLdr(vocab::TreeLdr::Self_))),
			metadata.clone(),
		));
		field
			.as_component_mut()
			.name_mut()
			.insert(Meta(treeldr::Name::new("id").unwrap(), metadata.clone()));
		field
			.as_formatted_mut()
			.format_mut()
			.insert(Meta(field_layout, metadata.clone()));
		let fields_id = self.create_list(
			vocabulary,
			generator,
			[Meta(id_field.into_term(), metadata.clone())],
		);
		let layout = self
			.get_mut(Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Resource))))
			.unwrap()
			.as_layout_mut();
		layout.set_fields(Meta(fields_id, metadata.clone()));

		// rdfs:Class
		self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Class))),
			metadata.clone(),
		);

		// rdfs:DataType
		let datatype = self.declare_type(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Datatype))),
			metadata.clone(),
		);
		datatype.as_type_mut().sub_class_of_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Class))).into(),
			metadata.clone(),
		));

		// rdfs:Literal
		self.declare_type(Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::Literal))), metadata);
	}
}
