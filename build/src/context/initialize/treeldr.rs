use locspan::Meta;
use treeldr::{vocab, IriIndex, metadata::Merge, Id};

use crate::Context;

impl<M> Context<M> {
	pub fn define_treeldr_types(&mut self, metadata: M)
	where
		M: Clone + Merge,
	{
		use treeldr::layout::Primitive;

		self.declare_property(
			Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Self_))),
			metadata.clone(),
		);
		let prop = self
			.get_mut(Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(
				vocab::TreeLdr::Self_,
			))))
			.unwrap()
			.as_property_mut();

		prop.range_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(vocab::Term::Rdfs(vocab::Rdfs::Resource))),
			metadata.clone()
		));

		self.declare_primitive_layout(Primitive::Boolean, metadata.clone());
		self.declare_primitive_layout(Primitive::Integer, metadata.clone());
		self.declare_primitive_layout(Primitive::UnsignedInteger, metadata.clone());
		self.declare_primitive_layout(Primitive::Float, metadata.clone());
		self.declare_primitive_layout(Primitive::Double, metadata.clone());
		self.declare_primitive_layout(Primitive::String, metadata.clone());
		self.declare_primitive_layout(Primitive::Time, metadata.clone());
		self.declare_primitive_layout(Primitive::Date, metadata.clone());
		self.declare_primitive_layout(Primitive::DateTime, metadata.clone());
		self.declare_primitive_layout(Primitive::Iri, metadata.clone());
		self.declare_primitive_layout(Primitive::Uri, metadata.clone());
		self.declare_primitive_layout(Primitive::Url, metadata);
	}
}