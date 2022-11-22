use locspan::{MaybeLocated, Span};
use rdf_types::Vocabulary;
use treeldr::{Id, IriIndex, BlankIdIndex, Multiple, Type};
use contextual::WithContext;
use crate::Property;

#[derive(Debug)]
pub struct NodeBindingTypeInvalid<M> {
	pub subject: Id,
	pub property: Property,
	pub object: Id,
	pub expected: Type,
	pub found: Multiple<Type, M>
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeBindingTypeInvalid<M> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("invalid {} value type for `{}`", self.property.name().with(vocab), self.subject.with(vocab))
	}
}