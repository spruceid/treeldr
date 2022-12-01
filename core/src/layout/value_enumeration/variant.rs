use locspan::Meta;

use crate::{MetaOption, Id};

pub struct ValueVariant;

pub struct Definition<M> {
	/// Layout representation.
	/// 
	/// For now, only the `tldr:String` layout is supported.
	representation: Meta<String, M>,

	/// Optional value.
	/// 
	/// If the value enumeration layout is not orphan, then this value is not
	/// `None`.
	value: MetaOption<Id, M>
}

impl<M> Definition<M> {
	pub fn new(
		representation: Meta<String, M>,
		value: MetaOption<Id, M>
	) -> Self {
		Self {
			representation,
			value
		}
	}

	pub fn representation(&self) -> &Meta<String, M> {
		&self.representation
	}

	pub fn value(&self) -> Option<&Meta<Id, M>> {
		self.value.as_ref()
	}
}