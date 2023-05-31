use locspan::{MaybeLocated, Span};
use rdf_types::Vocabulary;
use treeldr::{Id, IriIndex, BlankIdIndex, Name, PropertyValue, Value, value::Literal};
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
	Id(Id, PropertyValue<Id, M>),
	Value(Value, PropertyValue<Value, M>),
	Literal(Literal, PropertyValue<Literal, M>),
	Name(Name, PropertyValue<Name, M>),
	Boolean(bool, PropertyValue<bool, M>)
}

impl<M> From<(Id, PropertyValue<Id, M>)> for ConflictValues<M> {
	fn from((a, b): (Id, PropertyValue<Id, M>)) -> Self {
		Self::Id(a, b)
	}
}

impl<M> From<(Value, PropertyValue<Value, M>)> for ConflictValues<M> {
	fn from((a, b): (Value, PropertyValue<Value, M>)) -> Self {
		Self::Value(a, b)
	}
}

impl<M> From<(Literal, PropertyValue<Literal, M>)> for ConflictValues<M> {
	fn from((a, b): (Literal, PropertyValue<Literal, M>)) -> Self {
		Self::Literal(a, b)
	}
}

impl<M> From<(Name, PropertyValue<Name, M>)> for ConflictValues<M> {
	fn from((a, b): (Name, PropertyValue<Name, M>)) -> Self {
		Self::Name(a, b)
	}
}

impl<M> From<(bool, PropertyValue<bool, M>)> for ConflictValues<M> {
	fn from((a, b): (bool, PropertyValue<bool, M>)) -> Self {
		Self::Boolean(a, b)
	}
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeBindingFunctionalConflict<M> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("conflicting {} value for `{}`", self.property.name().with(vocab), self.id.with(vocab))
	}
}