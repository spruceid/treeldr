use core::fmt;
use std::{cmp::Ordering, fmt::Write, ops::Bound, sync::OnceLock};

use iregex::automata::{AnyRange, Automaton, RangeSet};

use crate::{display::DisplaySignature, utils::Ignored};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TextStringPrecomputedData {
	is_any: bool,
	regex_string: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextString {
	automaton: iregex::automata::NFA<u32>,
	singleton: Ignored<OnceLock<Option<String>>>,
	precomputed_data: TextStringPrecomputedData,
}

impl Default for TextString {
	fn default() -> Self {
		let mut any_char = RangeSet::new();
		any_char.insert(AnyRange::new(
			Bound::Included('\0'),
			Bound::Included('\u{D7FF}'),
		));
		any_char.insert(AnyRange::new(
			Bound::Included('\u{E000}'),
			Bound::Included('\u{10FFFF}'),
		));

		let mut automaton = iregex::automata::NFA::new();
		automaton.add_initial_state(0);
		automaton.add(0, Some(any_char), 0);
		automaton.add_final_state(0);

		Self {
			automaton,
			singleton: Ignored(OnceLock::new()),
			precomputed_data: TextStringPrecomputedData {
				is_any: true,
				regex_string: ".*".to_owned(),
			},
		}
	}
}

impl TextString {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn singleton(value: String) -> Self {
		let automaton = iregex::automata::NFA::singleton(value.chars(), |i| match i {
			Some(i) => i as u32 + 1,
			None => 0,
		});

		let mut regex_string = String::new();
		for c in value.chars() {
			match c {
				'(' => regex_string.push_str("\\("),
				')' => regex_string.push_str("\\)"),
				'[' => regex_string.push_str("\\["),
				']' => regex_string.push_str("\\]"),
				'\\' => regex_string.push_str("\\\\"),
				'.' => regex_string.push_str("\\."),
				'*' => regex_string.push_str("\\*"),
				'+' => regex_string.push_str("\\+"),
				'?' => regex_string.push_str("\\?"),
				'^' => regex_string.push_str("\\^"),
				c => regex_string.push(c),
			}
		}

		let singleton = OnceLock::new();
		let _ = singleton.set(Some(value));

		Self {
			automaton,
			singleton: Ignored(singleton),
			precomputed_data: TextStringPrecomputedData {
				is_any: false,
				regex_string,
			},
		}
	}

	pub fn is_singleton(&self) -> bool {
		self.automaton.is_singleton()
	}

	pub fn as_singleton(&self) -> Option<&str> {
		self.singleton
			.get_or_init(|| {
				self.automaton
					.to_singleton()
					.map(|chars| chars.into_iter().collect())
			})
			.as_deref()
	}

	pub fn automaton(&self) -> &iregex::automata::NFA<u32> {
		&self.automaton
	}

	pub fn set_automaton(&mut self, aut: iregex::automata::NFA<u32>) {
		self.automaton = aut;
		self.singleton = Ignored(OnceLock::new())
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self.as_singleton(), other.as_singleton()) {
			(Some(a), Some(b)) => {
				if a == b {
					Some(Ordering::Equal)
				} else {
					None
				}
			}
			(None, Some(b)) => {
				if self.automaton.contains(b.chars()) {
					Some(Ordering::Greater)
				} else {
					None
				}
			}
			(Some(a), None) => {
				if self.automaton.contains(a.chars()) {
					Some(Ordering::Less)
				} else {
					None
				}
			}
			(None, None) => {
				if self.automaton == other.automaton {
					Some(Ordering::Equal)
				} else {
					None
				}
			}
		}
	}

	pub fn contains(&self, value: &str) -> bool {
		match self.as_singleton() {
			Some(singleton) => singleton == value,
			None => self.automaton.contains(value.chars()),
		}
	}
}

impl<P, R> DisplaySignature<P, R> for TextString {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<
			impl crate::Registry<PackageId = P, Resource = R>,
		>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self.as_singleton() {
			Some(s) => {
				f.write_char('"')?;

				for c in s.chars() {
					match c {
						'\t' => f.write_str("\\t")?,
						'\r' => f.write_str("\\r")?,
						'\n' => f.write_str("\\n")?,
						'\'' => f.write_str("\\'")?,
						'\"' => f.write_str("\\\"")?,
						'\\' => f.write_str("\\\\")?,
						c => f.write_char(c)?,
					}
				}

				f.write_char('"')
			}
			None => {
				if self.precomputed_data.is_any {
					write!(f, "string")
				} else {
					write!(f, "/{}/", self.precomputed_data.regex_string)
				}
			}
		}
	}
}
