use std::collections::HashMap;
use super::Definition;
use crate::{Collection, Ref};

pub struct StronglyConnectedLayouts<'l> {
	layouts: &'l Collection<Definition>,
	map: HashMap<Ref<Definition>, u32>,
	component_count: u32
}

impl<'l> StronglyConnectedLayouts<'l> {
	#[inline(always)]
	pub fn new(layouts: &'l Collection<Definition>) -> Self {
		Self::from_entry_points(layouts, layouts.iter().map(|(layout_ref, _)| layout_ref))
	}

	#[inline(always)]
	pub fn from_entry_points<I: IntoIterator<Item=Ref<Definition>>>(layouts: &'l Collection<Definition>, entry_points: I) -> Self {
		Self::from_entry_points_with_filter(layouts, entry_points, |_, _| true)
	}

	#[inline(always)]
	pub fn with_filter(layouts: &'l Collection<Definition>, filter: impl Clone + Fn(Ref<Definition>, Ref<Definition>) -> bool) -> Self {
		Self::from_entry_points_with_filter(layouts, layouts.iter().map(|(layout_ref, _)| layout_ref), filter)
	}

	pub fn from_entry_points_with_filter<I: IntoIterator<Item=Ref<Definition>>>(layouts: &'l Collection<Definition>, entry_points: I, filter: impl Clone + Fn(Ref<Definition>, Ref<Definition>) -> bool) -> Self {
		let mut components = Self {
			layouts,
			map: HashMap::new(),
			component_count: 0
		};

		let mut map = HashMap::new();
		let mut stack = Vec::new();

		for layout_ref in entry_points {
			if !map.contains_key(&layout_ref) {
				strong_connect(&mut components, &mut map, &mut stack, layout_ref, filter.clone());
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

	#[inline(always)]
	pub fn is_recursive(&self, layout_ref: Ref<Definition>) -> Option<bool> {
		self.is_recursive_with_filter(layout_ref, |_| true)
	}

	pub fn is_recursive_with_filter(&self, layout_ref: Ref<Definition>, filter: impl Fn(Ref<Definition>) -> bool) -> Option<bool> {
		let layout = self.layouts.get(layout_ref)?;
		let component = self.component(layout_ref)?;

		for sub_layout_ref in layout.composing_layouts().into_iter().flatten() {
			if filter(sub_layout_ref) {
				if self.component(sub_layout_ref)? == component {
					return Some(true)
				}
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
	layout_ref: Ref<Definition>,
	filter: impl Clone + Fn(Ref<Definition>, Ref<Definition>) -> bool
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
	for sub_layout_ref in layout.composing_layouts().into_iter().flatten() {
		if filter(layout_ref, sub_layout_ref) {
			let new_layout_low_link = match map.get(&sub_layout_ref) {
				None => {
					let sub_layout_low_link = strong_connect(components, map, stack, sub_layout_ref, filter.clone());
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