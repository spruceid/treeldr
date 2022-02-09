use std::collections::HashMap;
use super::Definition;
use crate::{Collection, Ref};

pub struct StronglyConnectedLayouts<'l> {
	layouts: &'l Collection<Definition>,
	map: HashMap<Ref<Definition>, u32>,
	component_count: u32
}

impl<'l> StronglyConnectedLayouts<'l> {
	pub fn new(layouts: &'l Collection<Definition>) -> Self {
		Self::new_from(layouts, layouts.iter().map(|(layout_ref, _)| layout_ref))
	}

	pub fn new_from<I: IntoIterator<Item=Ref<Definition>>>(layouts: &'l Collection<Definition>, entry_points: I) -> Self {
		let mut components = Self {
			layouts,
			map: HashMap::new(),
			component_count: 0
		};

		let mut map = HashMap::new();
		let mut stack = Vec::new();

		for layout_ref in entry_points {
			if !map.contains_key(&layout_ref) {
				strong_connect(&mut components, &mut map, &mut stack, layout_ref);
			}
		}

		components
	}

	fn next(&mut self) -> u32 {
		let c = self.component_count;
		self.component_count += 1;
		c
	}

	fn set(&mut self, layout_ref: Ref<Definition>, component: u32) {
		self.map.insert(layout_ref, component);
	}

	pub fn component(&self, layout_ref: Ref<Definition>) -> Option<u32> {
		self.map.get(&layout_ref).cloned()
	}

	pub fn is_recursive(&self, layout_ref: Ref<Definition>) -> Option<bool> {
		let layout = self.layouts.get(layout_ref)?;
		let component = self.component(layout_ref)?;

		for sub_layout_expr in layout.composing_layouts().into_iter().flatten() {
			let sub_layout_ref = sub_layout_expr.layout();
			if self.component(sub_layout_ref)? == component {
				return Some(true)
			}
		}

		Some(false)
	}
}

struct Data {
	index: u32,
	low_link: u32,
	on_stack: bool
}

fn strong_connect<'l>(
	components: &mut StronglyConnectedLayouts<'l>,
	map: &mut HashMap<Ref<Definition>, Data>,
	stack: &mut Vec<Ref<Definition>>,
	layout_ref: Ref<Definition>
) -> u32 {
	let index = map.len() as u32;
	stack.push(layout_ref);
	map.insert(
		layout_ref,
		Data {
			index,
			low_link: index,
			on_stack: true
		}
	);

	let layout = components.layouts.get(layout_ref).unwrap();
	for sub_layout_expr in layout.composing_layouts().into_iter().flatten() {
		let sub_layout_ref = sub_layout_expr.layout();
		let new_layout_low_link = match map.get(&sub_layout_ref) {
			None => {
				let sub_layout_low_link = strong_connect(components, map, stack, sub_layout_ref);
				Some(std::cmp::min(map[&layout_ref].low_link, sub_layout_low_link))
			}
			Some(sub_layout_data) => {
				if sub_layout_data.on_stack {
					Some(std::cmp::min(map[&layout_ref].low_link, sub_layout_data.index))
				} else {
					None
				}
			}
		};

		if let Some(new_layout_low_link) = new_layout_low_link {
			map.get_mut(&layout_ref).unwrap().low_link = new_layout_low_link;
		}
	}

	let low_link = map[&layout_ref].low_link;

	if low_link == map[&layout_ref].index {
		let component = components.next();

		loop {
			let other_layout_ref = stack.pop().unwrap();
			map.get_mut(&other_layout_ref).unwrap().on_stack = false;

			// Add w to current strongly connected component
			components.set(other_layout_ref, component);

			if other_layout_ref == layout_ref {
				break;
			}
		}

		// Output the current strongly connected component
	}

	low_link
}