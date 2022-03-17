use std::collections::BTreeSet;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Block {
	/// Block content.
	data: String,

	/// Byte index of the end of the short description.
	short_end: usize,

	/// Byte index of the start of the long description.
	long_start: usize
}

impl Block {
	pub fn new(s: String) -> Self {
		let mut short_end = 0;
		let mut long_start = 0;

		enum State {
			Short,
			ShortNewline,
			Separation
		}

		let mut state = State::Short;

		for (i, c) in s.char_indices() {
			match state {
				State::Short => {
					if c == '\n' {
						state = State::ShortNewline;
						short_end = i;
					}

					long_start = i;
				}
				State::ShortNewline => {
					if c == '\n' {
						state = State::Separation;
					} else if !c.is_whitespace() {
						state = State::Short;
					}

					long_start = i;
				}
				State::Separation => {
					long_start = i;

					if !c.is_whitespace() {
						break
					}
				}
			}
		}

		Self {
			data: s,
			short_end,
			long_start
		}
	}

	pub fn short_description(&self) -> Option<&str> {
		let s = &self.data[..self.short_end];
		if s.trim().is_empty() {
			None
		} else {
			Some(s)
		}
	}

	pub fn long_description(&self) -> Option<&str> {
		let s = &self.data[self.long_start..];
		if s.trim().is_empty() {
			None
		} else {
			Some(s)
		}
	}
}

#[derive(Clone, Default, Debug)]
pub struct Documentation {
	blocks: BTreeSet<Block>
}

impl Documentation {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_empty(&self) -> bool {
		self.blocks.is_empty()
	}

	pub fn short_description(&self) -> Option<&str> {
		for block in &self.blocks {
			if let Some(s) = block.short_description() {
				return Some(s)
			}
		}

		None
	}

	pub fn long_description(&self) -> Option<&str> {
		for block in &self.blocks {
			if let Some(s) = block.long_description() {
				return Some(s)
			}
		}

		None
	}

	pub fn add(&mut self, comment: String) {
		self.blocks.insert(Block::new(comment));
	}
}
