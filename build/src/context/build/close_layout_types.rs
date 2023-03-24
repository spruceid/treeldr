use std::collections::HashMap;

use locspan::Meta;
use treeldr::{metadata::Merge, Id};

use crate::Context;

impl<M> Context<M> {
	/// Close the layout `rdf:layoutFor` relation by layout aliasing.
	pub(crate) fn close_layout_types(&mut self)
	where
		M: Clone + Merge,
	{
		let mut stack = Vec::new();

		for (_, node) in self.nodes() {
			for target in &node.as_layout().description().alias {
				let target = **target.value;
				for ty in node.as_layout().ty() {
					stack.push((target, ty.value.cloned()))
				}
			}
		}

		fn insert<M>(
			context: &Context<M>,
			diff: &mut HashMap<Id, HashMap<Id, M>>,
			layout_ref: Id,
			Meta(type_ref, meta): &Meta<Id, M>,
		) -> bool
		where
			M: Clone + Merge,
		{
			let node = context.get(layout_ref).unwrap();
			if !node.as_layout().ty().contains(type_ref) {
				let types = diff.entry(layout_ref).or_default();
				types.insert(*type_ref, meta.clone()).is_none()
			} else {
				false
			}
		}

		let mut diff = HashMap::new();

		while let Some((layout_ref, ty_ref)) = stack.pop() {
			if insert(self, &mut diff, layout_ref, &ty_ref) {
				let node = self.get(layout_ref).unwrap();
				for target in &node.as_layout().description().alias {
					let target = **target.value;
					stack.push((target, ty_ref.clone()))
				}
			}
		}

		for (layout_ref, diff) in diff {
			let node = self.get_mut(layout_ref).unwrap();
			node.as_layout_mut()
				.ty_mut()
				.extend(diff.into_iter().map(|(t, m)| Meta(t, m)))
		}
	}
}
