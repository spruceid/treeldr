use locspan::{MaybeLocated, Span, Meta};
use rdf_types::Vocabulary;
use treeldr::{Id, IriIndex, BlankIdIndex, Type, PropertyValues};
use contextual::WithContext;

use crate::{Property, Error};

use super::NodeBindingTypeInvalid;

#[derive(Debug)]
pub struct NodeTypeInvalid<M> {
	id: Id,
	expected: Type,
	found: PropertyValues<Type, M>
}

impl<M> NodeTypeInvalid<M> {
	pub fn new(
		id: Id,
		expected: Type,
		found: PropertyValues<Type, M>
	) -> Self {
		// panic!("invalid type");
		Self { id, expected, found }
	}
}

impl<M> NodeTypeInvalid<M> {
	pub fn at(self, meta: M) -> Error<M> {
		Meta(self.into(), meta)
	}

	pub fn for_node_binding(self, subject: Id, property: impl Into<Property>) -> NodeBindingTypeInvalid<M> {
		NodeBindingTypeInvalid {
			subject,
			property: property.into(),
			object: self.id,
			expected: self.expected,
			found: self.found
		}
	}
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeTypeInvalid<M> {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("invalid type for `{}`", self.id.with(vocab))
	}
}