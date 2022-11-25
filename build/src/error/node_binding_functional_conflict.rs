use locspan::{Meta, MaybeLocated, Span, Stripped};
use rdf_types::Vocabulary;
use treeldr::{Id, IriIndex, BlankIdIndex, Name, vocab::Object};
use contextual::WithContext;
use crate::Property;

/// Functional property conflict error.
/// 
/// This error is raised when two different values are given to a functional
/// property.
#[derive(Debug)]
pub struct NodeBindingFunctionalConflict<M> {
	pub id: Id,
	pub property: Property,
	pub values: ConflictValues<M>
}

#[derive(Debug)]
pub enum ConflictValues<M> {
	Id(Id, Meta<Id, M>),
	Object(Object<M>, Meta<Object<M>, M>),
	Name(Name, Meta<Name, M>),
	Boolean(bool, Meta<bool, M>)
}

impl<M> From<(Id, Meta<Id, M>)> for ConflictValues<M> {
	fn from((a, b): (Id, Meta<Id, M>)) -> Self {
		Self::Id(a, b)
	}
}

impl<M> From<(Object<M>, Meta<Object<M>, M>)> for ConflictValues<M> {
	fn from((a, b): (Object<M>, Meta<Object<M>, M>)) -> Self {
		Self::Object(a, b)
	}
}

impl<M> From<(Stripped<Object<M>>, Meta<Stripped<Object<M>>, M>)> for ConflictValues<M> {
	fn from((a, b): (Stripped<Object<M>>, Meta<Stripped<Object<M>>, M>)) -> Self {
		Self::Object(a.unwrap(), b.map(Stripped::unwrap))
	}
}

impl<M> From<(Name, Meta<Name, M>)> for ConflictValues<M> {
	fn from((a, b): (Name, Meta<Name, M>)) -> Self {
		Self::Name(a, b)
	}
}

impl<M> From<(bool, Meta<bool, M>)> for ConflictValues<M> {
	fn from((a, b): (bool, Meta<bool, M>)) -> Self {
		Self::Boolean(a, b)
	}
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeBindingFunctionalConflict<M> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("conflicting {} value for `{}`", self.property.name().with(vocab), self.id.with(vocab))
	}
}