use locspan::MapLocErr;
use std::cmp::Ordering;
use treeldr::{metadata::Merge, node, Id, Property, TId};

use crate::{error, prop::Hierarchy, Context, Error};

impl<M> Context<M> {
	fn dispatch_property(&self, prop_hierarchy: &Hierarchy<M>, prop: Id) -> Property {
		for built_in_prop in &node::Property::ALL {
			match prop_hierarchy.cmp(built_in_prop.id(), prop) {
				Some(Ordering::Equal) => return Property::Resource(*built_in_prop),
				Some(Ordering::Greater) => {
					log::debug!("{prop:?} is a sub property of {built_in_prop:?}");
					return Property::Resource(
						built_in_prop.into_sub_property(Some(TId::new(prop))),
					);
				}
				_ => (),
			}
		}

		Property::Other(TId::new(prop))
	}

	pub fn dispatch_sub_properties(&mut self) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		let hierarchy = Hierarchy::new(self).map_loc_err(error::Description::from)?;

		let mut nodes_other_properties = Vec::with_capacity(self.nodes.len());

		for node in self.nodes.values_mut() {
			nodes_other_properties.push(std::mem::take(node.other_properties_mut()))
		}

		let dispatched_nodes_other_properties: Vec<_> = nodes_other_properties
			.into_iter()
			.map(|other_properties| {
				let mut dispatched = Vec::new();

				for (base_prop, values) in other_properties {
					log::debug!("dispatching {base_prop:?}");
					for value in values {
						let prop = value.sub_property.map(TId::into_id).unwrap_or(base_prop);
						dispatched.push((self.dispatch_property(&hierarchy, prop), value.value));
					}
				}

				dispatched
			})
			.collect();

		for (node, other_properties) in self
			.nodes
			.values_mut()
			.zip(dispatched_nodes_other_properties)
		{
			for (prop, value) in other_properties {
				node.set(prop, |a, b| hierarchy.cmp(a.id(), b.id()), value)?;
			}
		}

		Ok(())
	}
}
