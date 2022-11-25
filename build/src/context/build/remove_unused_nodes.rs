use std::collections::HashSet;

use locspan::Meta;
use treeldr::Id;

use crate::Context;

impl<M> Context<M> {
	pub fn remove_unused_nodes(&mut self) {
		let mut used_nodes = HashSet::new();
		let mut stack = Vec::new();

		for id in self.nodes.keys() {
			if let Id::Iri(_) = id {
				stack.push(*id);
			}
		}

		while let Some(id) = stack.pop() {
			if used_nodes.insert(id) {
				if let Some(node) = self.get(id) {
					for Meta(b, _) in node.bindings() {
						if let Some(used_id) = b.value().into_id() {
							stack.push(used_id)
						}
					}
				}
			}
		}

		let filtered_nodes = std::mem::take(&mut self.nodes)
			.into_iter()
			.filter(|(id, _)| used_nodes.contains(id));
		self.nodes.extend(filtered_nodes);
	}
}
