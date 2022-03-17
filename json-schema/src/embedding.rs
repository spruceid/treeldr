use std::collections::HashMap;
use treeldr::{layout, Ref};
use derivative::Derivative;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Embedding {
	Reference,
	Direct,
	Indirect,
}

impl Embedding {
	pub fn is_direct(&self) -> bool {
		matches!(self, Self::Direct)
	}

	pub fn is_indirect(&self) -> bool {
		matches!(self, Self::Indirect)
	}
}

impl Default for Embedding {
	fn default() -> Self {
		Self::Reference
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Configuration<F> {
	map: HashMap<Ref<layout::Definition<F>>, Embedding>,
}

impl<F> Configuration<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, layout_ref: Ref<layout::Definition<F>>) -> Embedding {
		self.map.get(&layout_ref).cloned().unwrap_or_default()
	}

	pub fn set(&mut self, layout_ref: Ref<layout::Definition<F>>, e: Embedding) {
		self.map.insert(layout_ref, e);
	}

	pub fn indirect_layouts(&self) -> impl '_ + Iterator<Item = Ref<layout::Definition<F>>> {
		self.map
			.iter()
			.filter_map(|(r, e)| if e.is_indirect() { Some(*r) } else { None })
	}
}
