use derivative::Derivative;
use std::collections::HashMap;
use treeldr::TId;

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
pub struct Configuration {
	map: HashMap<TId<treeldr::Layout>, Embedding>,
}

impl Configuration {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, layout_ref: TId<treeldr::Layout>) -> Embedding {
		self.map.get(&layout_ref).cloned().unwrap_or_default()
	}

	pub fn set(&mut self, layout_ref: TId<treeldr::Layout>, e: Embedding) {
		self.map.insert(layout_ref, e);
	}

	pub fn indirect_layouts(&self) -> impl '_ + Iterator<Item = TId<treeldr::Layout>> {
		self.map
			.iter()
			.filter_map(|(r, e)| if e.is_indirect() { Some(*r) } else { None })
	}
}
