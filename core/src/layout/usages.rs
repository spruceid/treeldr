use std::collections::{HashMap, HashSet};

use crate::{Layout, MutableModel, TId};

pub struct Usages {
	map: HashMap<TId<Layout>, HashSet<TId<Layout>>>,
}

impl Usages {
	pub fn new<M>(model: &MutableModel<M>) -> Self {
		use std::collections::hash_map::Entry;
		let mut map: HashMap<TId<Layout>, HashSet<TId<Layout>>> = HashMap::new();

		for (layout_ref, layout) in model.layouts() {
			if let Entry::Vacant(entry) = map.entry(layout_ref) {
				entry.insert(HashSet::new());
			}

			for sub_layout_ref in layout.as_layout().composing_layouts(model) {
				match map.entry(sub_layout_ref) {
					Entry::Occupied(mut entry) => {
						entry.get_mut().insert(layout_ref);
					}
					Entry::Vacant(entry) => {
						entry.insert(Some(layout_ref).into_iter().collect());
					}
				}
			}
		}

		Self { map }
	}

	pub fn get(&self, layout_ref: TId<Layout>) -> Option<&HashSet<TId<Layout>>> {
		self.map.get(&layout_ref)
	}
}
