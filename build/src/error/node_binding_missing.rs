use locspan::{MaybeLocated, Span};
use treeldr::{Id, vocab::TldrVocabulary};
use contextual::WithContext;
use crate::Property;

#[derive(Debug)]
pub struct NodeBindingMissing {
	pub id: Id,
	pub property: Property
}

impl NodeBindingMissing {
	pub fn new(
		id: Id,
		property: impl Into<Property>
	) -> Self {
		Self {
			id, property: property.into()
		}
	}
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeBindingMissing {
	fn message(&self, vocab: &TldrVocabulary) -> String {
		format!("missing {} for `{}`", self.property.name().with(vocab), self.id.with(vocab))
	}
}