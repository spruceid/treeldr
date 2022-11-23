use std::collections::HashMap;

use treeldr::{Id, metadata::Merge};

use crate::{Context, ListRef, ObjectAsId, context::MapIds};

impl<M: Merge> Context<M> {
	pub fn simplify_composite_layouts(&mut self) {
		let mut map = HashMap::new();
		for (id, node) in &self.nodes {
			if let Id::Blank(id) = id {
				if !node.as_layout().intersection_of().len() == 1 {
					let list_id = **node.as_layout().intersection_of().first().unwrap();
					if let Some(ListRef::Cons(_, d, _)) = self.get_list(list_id) {
						if d.first().len() == 1 && d.rest().len() == 1 {
							if let Some(ListRef::Nil) = self.get_list(**d.rest().first().unwrap()) {
								if let Some(first) = d.first().first().unwrap().as_id() {
									map.insert(*id, first);
								}
							}
						}
					}
				}
			}
		}

		self.map_ids(|id| {
			match id {
				Id::Iri(i) => Id::Iri(i),
				Id::Blank(b) => map.get(&b).copied().unwrap_or(Id::Blank(b))
			}
		})
	}
}