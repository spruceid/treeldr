use locspan::{MaybeLocated, Span};
use rdf_types::Vocabulary;
use treeldr::{Id, IriIndex, BlankIdIndex, Multiple, Type};
use contextual::WithContext;

use crate::node;

use super::NodeBindingTypeInvalid;

#[derive(Debug)]
pub struct NodeTypeInvalid<M> {
	pub id: Id,
	pub expected: Type,
	pub found: Multiple<Type, M>
}

impl<M> NodeTypeInvalid<M> {
	pub fn for_node_binding(self, subject: Id, property: impl Into<node::Property>) -> NodeBindingTypeInvalid<M> {
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