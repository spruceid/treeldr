use super::Properties;
use crate::{prop, Causes, Model, Ref};
use once_cell::unsync::OnceCell;
use std::collections::HashMap;

/// Intersection type.
pub struct Intersection<F> {
	/// Types in the intersection.
	types: HashMap<Ref<super::Definition<F>>, Causes<F>>,

	/// Properties in the intersection.
	///
	/// Lazily computed.
	properties: OnceCell<HashMap<Ref<prop::Definition<F>>, Causes<F>>>,
}

impl<F> Intersection<F> {
	pub fn new(types: HashMap<Ref<super::Definition<F>>, Causes<F>>) -> Self {
		Self {
			types,
			properties: OnceCell::new(),
		}
	}

	pub fn properties<'m>(&'m self, model: &'m Model<F>) -> Properties<'m, F>
	where
		F: Clone + Ord,
	{
		// Compute the properties in the intersection if not already.
		let properties = self.properties.get_or_init(|| {
			let mut properties = None;

			fn intersection<F: Clone + Ord>(
				current: Option<HashMap<Ref<prop::Definition<F>>, Causes<F>>>,
				other: super::Properties<F>,
			) -> HashMap<Ref<prop::Definition<F>>, Causes<F>> {
				match current {
					Some(current) => other
						.filter_map(|(p, cb)| {
							current.get(&p).map(|ca| {
								let mut causes = ca.clone();
								causes.extend(cb.clone());
								(p, causes)
							})
						})
						.collect(),
					None => other.map(|(p, c)| (p, c.clone())).collect(),
				}
			}

			for ty_ref in self.types.keys() {
				let ty = model.types().get(*ty_ref).unwrap();
				properties = Some(intersection(properties, ty.properties(model)));
			}

			properties.unwrap_or_default()
		});

		Properties(properties.iter())
	}
}
