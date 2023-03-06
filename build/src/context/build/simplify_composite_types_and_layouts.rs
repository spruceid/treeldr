use std::collections::HashMap;

use locspan::Meta;
use treeldr::{metadata::Merge, Id, ResourceType, Value};

use crate::{context::MapIds, layout, Context, FunctionalPropertyValue, ListRef};

impl<M: Merge> Context<M> {
	pub fn simplify_composite_types_and_layouts(&mut self) -> bool {
		let mut layout_map = HashMap::new();
		let mut type_map = HashMap::new();

		for (id, node) in &self.nodes {
			if let Id::Blank(id) = id {
				if node.as_type().intersection_of().is_empty() {
					if let Some(s) = get_singleton(self, node.as_type().union_of()) {
						type_map.insert(*id, s);
					}
				} else if node.as_type().union_of().is_empty() {
					if let Some(s) = get_singleton(self, node.as_type().intersection_of()) {
						type_map.insert(*id, s);
					}
				}

				match get_singleton(self, node.as_layout().intersection_of()) {
					Some(s) => {
						layout_map.insert(*id, s);
					}
					None => {
						if let Some(b) = get_best_intersection_id(
							self,
							node.as_layout().description(),
							node.as_layout().intersection_of(),
						) {
							layout_map.insert(*id, b);
						}
					}
				}
			}
		}

		let changed = !type_map.is_empty() || !layout_map.is_empty();

		for (b, target) in &type_map {
			let node = self.nodes.get_mut(&Id::Blank(*b)).unwrap();
			node.as_type_mut().union_of_mut().clear();
			node.as_type_mut().intersection_of_mut().clear();

			if *target != Id::Blank(*b) {
				node.type_mut().remove(&crate::Type::TYPE);
			}
		}

		for (b, target) in &layout_map {
			let node = self.nodes.get_mut(&Id::Blank(*b)).unwrap();
			node.as_layout_mut().intersection_of_mut().clear();

			if *target != Id::Blank(*b) {
				node.type_mut().remove(&treeldr::Layout::TYPE);
			}
		}

		self.map_ids(|id, prop| match id {
			Id::Iri(i) => Id::Iri(i),
			Id::Blank(b) => match prop {
				Some(prop) if prop.expect_type() => {
					type_map.get(&b).copied().unwrap_or(Id::Blank(b))
				}
				Some(prop) if prop.expect_layout() => {
					layout_map.get(&b).copied().unwrap_or(Id::Blank(b))
				}
				_ => Id::Blank(b),
			},
		});

		changed
	}
}

fn get_singleton<M>(context: &Context<M>, list: &FunctionalPropertyValue<Id, M>) -> Option<Id> {
	if list.len() == 1 {
		let mut result = None;
		let mut list_id = **list.first().unwrap().value;

		loop {
			match context.get_list(list_id) {
				Some(ListRef::Nil) => break result,
				Some(ListRef::Cons(_, d, _)) => {
					if d.first().len() == 1 && d.rest().len() == 1 {
						list_id = **d.rest().first().unwrap().value;
						if let Some(first) = d.first().first().unwrap().value.as_id() {
							if result.is_none() || result == Some(first) {
								result = Some(first)
							} else {
								break None;
							}
						} else {
							break None;
						}
					} else {
						break None;
					}
				}
				None => break None,
			}
		}
	} else {
		None
	}
}

fn get_best_intersection_id<M>(
	context: &Context<M>,
	desc: &layout::DescriptionProperties<M>,
	list_value: &FunctionalPropertyValue<Id, M>,
) -> Option<Id> {
	let mut candidate = None;

	for list_id in list_value {
		let list = context.get_list(**list_id.value).unwrap();

		for item in list.lenient_iter(context) {
			match item {
				Meta(Value::Node(id), _) => {
					let item = context.get(*id).unwrap();
					if item
						.as_layout()
						.description()
						.is_equivalent_to(context, desc)
					{
						if let Some(old_candidate) = candidate.replace(*id) {
							if old_candidate != *id {
								return None;
							}
						}
					}
				}
				Meta(Value::Literal(_), _) => {
					return None;
				}
			}
		}
	}

	candidate
}
