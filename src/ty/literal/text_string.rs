use core::fmt;
use std::{cmp::Ordering, fmt::Write};

use iregex::automata::{any_char, Automaton};
use serde_seeded::{unseeded, DeserializeSeeded, SerializeSeeded};

use crate::{display::DisplaySignature, utils::Ignored};

#[derive(Debug, Clone)]
struct PrecomputedData {
	singleton: Option<String>,
	is_any: bool,
	regex_string: String,
}

impl PrecomputedData {
	pub fn new(automaton: &iregex::automata::NFA<u32>) -> Self {
		match automaton.to_singleton() {
			Some(singleton) => Self::new_singleton(singleton.into_iter().collect()),
			None => {
				if automaton.is_universal(any_char()) {
					Self::new_any()
				} else {
					todo!()
				}
			}
		}
	}

	pub fn new_any() -> Self {
		Self {
			singleton: None,
			is_any: true,
			regex_string: ".*".to_owned(),
		}
	}

	pub fn new_singleton(singleton: String) -> Self {
		let mut regex_string = String::new();
		for c in singleton.chars() {
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

		Self {
			singleton: Some(singleton),
			is_any: false,
			regex_string,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded)]
#[seeded(ser(params(Q), seed(Q)))]
pub struct TextString {
	#[seeded(with(unseeded))]
	automaton: iregex::automata::NFA<u32>,

	#[seeded(skip)]
	precomputed_data: Ignored<PrecomputedData>,
}

impl Default for TextString {
	fn default() -> Self {
		Self {
			automaton: iregex::automata::NFA::simple_loop(0, any_char()),
			precomputed_data: Ignored(PrecomputedData::new_any()),
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

		Self {
			automaton,
			precomputed_data: Ignored(PrecomputedData::new_singleton(value)),
		}
	}

	pub fn is_singleton(&self) -> bool {
		self.automaton.is_singleton()
	}

	pub fn as_singleton_value(&self) -> Option<&str> {
		self.precomputed_data.singleton.as_deref()
	}

	pub fn to_singleton_value(&self) -> Option<String> {
		self.as_singleton_value().map(ToOwned::to_owned)
	}

	pub fn automaton(&self) -> &iregex::automata::NFA<u32> {
		&self.automaton
	}

	pub fn set_automaton(&mut self, aut: iregex::automata::NFA<u32>) {
		self.precomputed_data = Ignored(PrecomputedData::new(&aut));
		self.automaton = aut;
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self.as_singleton_value(), other.as_singleton_value()) {
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
		match self.as_singleton_value() {
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
		match self.as_singleton_value() {
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

impl<'de, Q> DeserializeSeeded<'de, Q> for TextString {
	fn deserialize_seeded<D>(seed: &Q, deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		#[derive(DeserializeSeeded)]
		#[seeded(de(params(Q), seed(Q)))]
		pub struct Inner {
			#[seeded(with(unseeded))]
			automaton: iregex::automata::NFA<u32>,
		}

		Inner::deserialize_seeded(seed, deserializer).map(|inner| {
			let precomputed_data = PrecomputedData::new(&inner.automaton);

			Self {
				automaton: inner.automaton,
				precomputed_data: Ignored(precomputed_data),
			}
		})
	}
}
