use std::{borrow::Cow, collections::BTreeSet, ops::Deref};

use locspan::Meta;

use crate::{metadata::Merge, property_values, value::Literal, PropertyValueRef, PropertyValues};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Block {
	/// Block content.
	data: Literal,

	/// Lexical form (if not stored in `data`).
	lexical_form: Option<String>,

	/// Byte index of the end of the short description.
	short_end: usize,

	/// Byte index of the start of the long description.
	long_start: usize,
}

impl Block {
	pub fn new(literal: Literal) -> Self {
		let mut lexical_form = None;

		let s = match literal.lexical_form() {
			Cow::Borrowed(s) => s,
			Cow::Owned(s) => {
				lexical_form = Some(s);
				lexical_form.as_ref().unwrap().as_str()
			}
		};

		let mut short_end = 0;
		let mut long_start = 0;

		#[derive(PartialEq, Eq)]
		enum State {
			Short,
			ShortNewline,
			Separation,
		}

		let mut state = State::Short;

		for (i, c) in s.char_indices() {
			match state {
				State::Short => {
					short_end = i;
					long_start = i;

					if c == '\n' {
						state = State::ShortNewline;
					}
				}
				State::ShortNewline => {
					if c == '\n' {
						state = State::Separation;
					} else if !c.is_whitespace() {
						short_end = i;
						state = State::Short;
					}

					long_start = i;
				}
				State::Separation => {
					long_start = i;

					if !c.is_whitespace() {
						break;
					}
				}
			}
		}

		if state == State::Short {
			short_end = s.len();
			long_start = s.len();
		}

		Self {
			data: literal,
			lexical_form,
			short_end,
			long_start,
		}
	}

	pub fn short_description(&self) -> Option<&str> {
		let s = &self.as_str()[..self.short_end];
		if s.trim().is_empty() {
			None
		} else {
			Some(s)
		}
	}

	pub fn long_description(&self) -> Option<&str> {
		let s = &self.as_str()[self.long_start..];
		if s.trim().is_empty() {
			None
		} else {
			Some(s)
		}
	}

	pub fn literal(&self) -> &Literal {
		&self.data
	}

	pub fn as_str(&self) -> &str {
		self.lexical_form
			.as_deref()
			.unwrap_or_else(|| match self.data.lexical_form() {
				Cow::Borrowed(s) => s,
				Cow::Owned(_) => panic!("no lexical form found"),
			})
	}
}

impl Deref for Block {
	type Target = Literal;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl From<String> for Block {
	fn from(value: String) -> Self {
		Self::new(Literal::String(value))
	}
}

impl<'a> From<&'a str> for Block {
	fn from(value: &'a str) -> Self {
		Self::new(Literal::String(value.to_owned()))
	}
}

#[derive(Clone, Debug)]
pub struct Documentation<M> {
	blocks: PropertyValues<Block, M>,
}

impl<M> Default for Documentation<M> {
	fn default() -> Self {
		Self {
			blocks: PropertyValues::default(),
		}
	}
}

impl<M> Documentation<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_comments(comments: PropertyValues<Block, M>) -> Self {
		Self { blocks: comments }
	}

	pub fn is_empty(&self) -> bool {
		self.blocks.is_empty()
	}

	pub fn short_description(&self) -> Option<&str> {
		for block in &self.blocks {
			if let Some(s) = block.value.short_description() {
				return Some(s);
			}
		}

		None
	}

	pub fn long_description(&self) -> Option<&str> {
		for block in &self.blocks {
			if let Some(s) = block.value.long_description() {
				return Some(s);
			}
		}

		None
	}

	pub fn insert(&mut self, Meta(comment, meta): Meta<Literal, M>)
	where
		M: Merge,
	{
		self.blocks.insert_base(Meta(Block::new(comment), meta));
	}

	pub fn as_str(&self) -> Option<&str> {
		self.blocks.iter().next().map(|b| b.value.as_str())
	}

	pub fn iter(&self) -> Iter<M> {
		self.blocks.iter()
	}

	pub fn clone_stripped(&self) -> StrippedDocumentation {
		let mut result = StrippedDocumentation::new();

		for PropertyValueRef {
			value: Meta(b, _), ..
		} in self.iter()
		{
			result.blocks.insert(b.clone());
		}

		result
	}
}

pub type Iter<'a, M> = property_values::non_functional::Iter<'a, Block, M>;

impl<'a, M> IntoIterator for &'a Documentation<M> {
	type Item = PropertyValueRef<'a, Block, M>;
	type IntoIter = Iter<'a, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

#[derive(Clone, Default, Debug)]
pub struct StrippedDocumentation {
	blocks: BTreeSet<Block>,
}

impl StrippedDocumentation {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_empty(&self) -> bool {
		self.blocks.is_empty()
	}

	pub fn short_description(&self) -> Option<&str> {
		for block in &self.blocks {
			if let Some(s) = block.short_description() {
				return Some(s);
			}
		}

		None
	}

	pub fn long_description(&self) -> Option<&str> {
		for block in &self.blocks {
			if let Some(s) = block.long_description() {
				return Some(s);
			}
		}

		None
	}

	pub fn insert(&mut self, comment: Literal) {
		self.blocks.insert(Block::new(comment));
	}

	pub fn as_str(&self) -> Option<&str> {
		self.blocks.iter().next().map(|b| b.as_str())
	}

	pub fn iter(&self) -> std::collections::btree_set::Iter<Block> {
		self.blocks.iter()
	}
}
