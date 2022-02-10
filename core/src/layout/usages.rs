use std::collections::{HashMap, HashSet};
use super::Definition;
use crate::{Collection, Ref};

pub struct Usages {
	map: HashMap<Ref<Definition>, HashSet<Ref<Definition>>>
}

impl Usages {
	pub fn new(layouts: &Collection<Definition>) -> Self {
		use std::collections::hash_map::Entry;
		let mut map: HashMap<Ref<Definition>, HashSet<Ref<Definition>>> = HashMap::new();

		for (layout_ref, layout) in layouts.iter() {
			if let Entry::Vacant(entry) = map.entry(layout_ref) {
				entry.insert(HashSet::new());
			}

			for sub_layout_ref in layout.composing_layouts().into_iter().flatten() {
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

		Self {
			map
		}
	}

	pub fn get(&self, layout_ref: Ref<Definition>) -> Option<&HashSet<Ref<Definition>>> {
		self.map.get(&layout_ref)
	}
}