use locspan::Meta;
use treeldr::{metadata::Merge, vocab, Id, IriIndex};

use crate::{layout, Context};

impl<M> Context<M> {
	pub fn define_xsd_types(&mut self, metadata: M)
	where
		M: Clone + Merge,
	{
		use vocab::{Term, Xsd};
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Boolean))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Date))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::DateTime))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Decimal))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Double))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Duration))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Float))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Int))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Integer))),
			metadata.clone(),
		);
		self.declare_datatype(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::String))),
			metadata.clone(),
		);
		let layout = self.declare_layout(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::String))),
			metadata.clone(),
		);
		layout.as_layout_mut().ty_mut().insert(Meta(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::String))),
			metadata.clone(),
		));
		layout
			.as_layout_mut()
			.set_alias(Meta(layout::Primitive::String.id(), metadata.clone()));

		self.declare_datatype(Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Time))), metadata);
	}
}
