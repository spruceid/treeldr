use super::Properties;
use crate::{prop, Causes, Model, Ref};
use once_cell::unsync::OnceCell;
use std::collections::HashMap;

pub struct Union<F> {
	options: HashMap<Ref<super::Definition<F>>, Causes<F>>,

	/// Properties in the union.
	///
	/// Lazily computed.
	properties: OnceCell<HashMap<Ref<prop::Definition<F>>, Causes<F>>>,
}

impl<F> Union<F> {
	pub fn new(options: HashMap<Ref<super::Definition<F>>, Causes<F>>) -> Self {
		Self {
			options,
			properties: OnceCell::new(),
		}
	}

	pub fn properties<'m>(&'m self, model: &'m Model<F>) -> Properties<'m, F>
	where
		F: Clone + Ord,
	{
		// Compute the properties in the intersection if not already.
		let properties = self.properties.get_or_init(|| {
			use std::collections::hash_map::Entry;
			let mut properties = HashMap::new();

			for ty_ref in self.options.keys() {
				let ty = model.types().get(*ty_ref).unwrap();
				for (prop, causes) in ty.properties(model) {
					match properties.entry(prop) {
						Entry::Vacant(entry) => {
							entry.insert(causes.clone());
						}
						Entry::Occupied(mut entry) => {
							entry.get_mut().extend(causes.clone());
						}
					}
				}
			}

			properties
		});

		Properties(properties.iter())
	}
}
