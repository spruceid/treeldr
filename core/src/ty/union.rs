use super::Properties;
use crate::{Causes, Ref};
use std::collections::BTreeMap;

pub struct Union<F> {
	options: BTreeMap<Ref<super::Definition<F>>, Causes<F>>,

	/// Properties in the union.
	properties: Properties<F>,
}

impl<F> Union<F> {
	pub fn new<'a, G>(options: BTreeMap<Ref<super::Definition<F>>, Causes<F>>, get: G) -> Self
	where
		F: 'a + Clone + Ord,
		G: 'a + Fn(Ref<super::Definition<F>>) -> &'a super::Definition<F>,
	{
		let mut properties = Properties::none();
		for &ty_ref in options.keys() {
			if let Some(ty_properties) = get(ty_ref).properties() {
				properties.unite_with(ty_properties);
			}
		}

		Self {
			options,
			properties,
		}
	}

	pub fn options(&self) -> impl '_ + DoubleEndedIterator<Item = Ref<super::Definition<F>>> {
		self.options.keys().cloned()
	}

	pub fn properties(&self) -> &Properties<F> {
		&self.properties
	}
}
