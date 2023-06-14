use locspan::{MaybeLocated, Span};
use treeldr::{Id, Type, PropertyValues, vocab::TldrVocabulary};
use contextual::WithContext;
use crate::Property;

#[derive(Debug)]
pub struct NodeBindingTypeInvalid<M> {
	pub subject: Id,
	pub property: Property,
	pub object: Id,
	pub expected: Type,
	pub found: PropertyValues<Type, M>
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeBindingTypeInvalid<M> {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("invalid {} value type for `{}`", self.property.name().with(vocab), self.subject.with(vocab))
	}
}