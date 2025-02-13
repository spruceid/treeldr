use btree_range_map::RangeSet;
use json_syntax::TryFromJson;
use std::{fmt, str::FromStr};

use crate::utils::{Automaton, DetAutomaton};

use super::syntax::{expect_string, Error};

impl TryFromJson for RegExp {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		_code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		let value = expect_string(json, offset)?;
		Self::parse(value).map_err(|e| Error::InvalidRegex(offset, e))
	}
}

/// Regular expression.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RegExp {
	/// Any character.
	///
	/// `.`
	Any,

	/// Character set.
	///
	/// `[]` or `[^ ]`
	Set(RangeSet<char>),

	/// Sequence.
	Sequence(Vec<Self>),

	/// Repetition.
	Repeat(Box<Self>, u32, u32),

	/// Union.
	Union(Vec<Self>),
}

impl RegExp {
	pub fn empty() -> Self {
		Self::Sequence(Vec::new())
	}

	/// Push the given regexp `e` at the end.
	///
	/// Builds the regexp sequence `self` followed by `e`.
	/// For instance if `self` is `/ab|cd/` then the result is `/(ab|cd)e/`
	pub fn push(&mut self, e: Self) {
		let this = match unsafe { std::ptr::read(self) } {
			Self::Sequence(mut seq) => {
				if seq.is_empty() {
					e
				} else {
					seq.push(e);
					Self::Sequence(seq)
				}
			}
			Self::Union(items) if items.is_empty() => e,
			item => Self::Sequence(vec![item, e]),
		};

		unsafe { std::ptr::write(self, this) }
	}

	pub fn repeat(&mut self, min: u32, max: u32) {
		let this = Self::Repeat(Box::new(unsafe { std::ptr::read(self) }), min, max);
		unsafe { std::ptr::write(self, this) }
	}

	pub fn simplified(self) -> Self {
		match self {
			Self::Any => Self::Any,
			Self::Set(set) => Self::Set(set),
			Self::Sequence(seq) => {
				let new_seq = seq
					.into_iter()
					.filter_map(|e| {
						if e.is_empty() {
							None
						} else {
							Some(e.simplified())
						}
					})
					.collect();
				Self::Sequence(new_seq)
			}
			Self::Union(items) => {
				if items.len() == 1 {
					items.into_iter().next().unwrap().simplified()
				} else {
					Self::Union(items.into_iter().map(Self::simplified).collect())
				}
			}
			Self::Repeat(e, min, max) => Self::Repeat(Box::new(e.simplified()), min, max),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::Set(set) => set.is_empty(),
			Self::Sequence(seq) => seq.iter().all(Self::is_empty),
			Self::Union(items) => items.iter().all(Self::is_empty),
			Self::Repeat(r, min, max) => r.is_empty() || (*min == 0 && *max == 0),
			_ => false,
		}
	}

	pub fn is_simple(&self) -> bool {
		matches!(self, Self::Any | Self::Set(_) | Self::Sequence(_))
	}

	/// Checks if this regular expression matches only one value.
	pub fn is_singleton(&self) -> bool {
		match self {
			Self::Any => false,
			Self::Set(charset) => charset.len() == 1,
			Self::Sequence(seq) => seq.iter().all(Self::is_singleton),
			Self::Repeat(e, min, max) => min == max && e.is_singleton(),
			Self::Union(items) => items.len() == 1 && items[0].is_singleton(),
		}
	}

	fn build_singleton(&self, s: &mut String) {
		match self {
			Self::Any => unreachable!(),
			Self::Set(charset) => s.push(charset.iter().next().unwrap().first().unwrap()),
			Self::Sequence(seq) => {
				for e in seq {
					e.build_singleton(s)
				}
			}
			Self::Repeat(e, _, _) => e.build_singleton(s),
			Self::Union(items) => items[0].build_singleton(s),
		}
	}

	pub fn as_singleton(&self) -> Option<String> {
		if self.is_singleton() {
			let mut s = String::new();
			self.build_singleton(&mut s);
			Some(s)
		} else {
			None
		}
	}

	/// Display this regular expression as a sub expression.
	///
	/// This will enclose it between parenthesis if necessary.
	pub fn display_sub(&self) -> DisplaySub {
		DisplaySub(self)
	}

	pub fn parse(s: &str) -> Result<Self, ParseError> {
		let mut stack = vec![vec![RegExp::empty()]];
		let mut chars = s.chars();

		while let Some(c) = chars.next() {
			match c {
				'(' => {
					stack.push(vec![RegExp::empty()]);
				}
				')' => {
					let sub_exp = RegExp::Union(stack.pop().unwrap()).simplified();
					let options = stack
						.last_mut()
						.ok_or(ParseError::UnmatchedClosingParenthesis)?;
					options.last_mut().unwrap().push(sub_exp);
				}
				'|' => {
					let options = stack.last_mut().unwrap();
					options.push(RegExp::empty());
				}
				'[' => {
					let options = stack.last_mut().unwrap();
					let charset = parse_charset(&mut chars)?;
					options.last_mut().unwrap().push(RegExp::Set(charset))
				}
				'\\' => {
					let options = stack.last_mut().unwrap();
					let c = parse_escaped_char(&mut chars)?;
					let mut charset = RangeSet::new();
					charset.insert(c);
					options.last_mut().unwrap().push(RegExp::Set(charset))
				}
				'?' => {
					let options = stack.last_mut().unwrap();
					options.last_mut().unwrap().repeat(0, 1)
				}
				'*' => {
					let options = stack.last_mut().unwrap();
					options.last_mut().unwrap().repeat(0, u32::MAX)
				}
				'+' => {
					let options = stack.last_mut().unwrap();
					options.last_mut().unwrap().repeat(1, u32::MAX)
				}
				c => {
					let options = stack.last_mut().unwrap();
					let mut charset = RangeSet::new();
					charset.insert(c);
					options.last_mut().unwrap().push(RegExp::Set(charset))
				}
			}
		}

		match stack.len() {
			0 => unreachable!(),
			1 => Ok(RegExp::Union(stack.into_iter().next().unwrap()).simplified()),
			_ => Err(ParseError::MissingClosingParenthesis),
		}
	}

	pub fn build(&self) -> DetAutomaton<usize> {
		let nd = self.build_non_deterministic();
		let dt = nd.determinize();
		debug_assert!(!dt.final_states().is_empty());

		let mut n = 0;
		dt.map(
			|_| {
				let r = n;
				n += 1;
				r
			},
			|label| *label,
		)
	}

	fn build_non_deterministic(&self) -> Automaton<usize> {
		let mut result = Automaton::new();

		let mut n = 0;
		let mut new_state = move || {
			let r = n;
			n += 1;
			r
		};

		let (a, b) = self.build_into(&mut new_state, &mut result);
		result.add_initial_state(a);
		result.add_final_state(b);
		debug_assert!(!result.final_states().is_empty());

		result
	}

	fn build_into(
		&self,
		new_state: &mut impl FnMut() -> usize,
		automaton: &mut Automaton<usize>,
	) -> (usize, usize) {
		match self {
			Self::Any => {
				let mut charset = RangeSet::new();
				charset.insert('\u{0}'..='\u{d7ff}');
				charset.insert('\u{e000}'..='\u{10ffff}');
				let a = new_state();
				let b = new_state();
				automaton.add(a, Some(charset), b);
				(a, b)
			}
			Self::Repeat(exp, min, max) => exp.build_repeat_into(new_state, automaton, *min, *max),
			Self::Sequence(exps) => {
				let mut a = new_state();

				for e in exps {
					let (ea, eb) = e.build_into(new_state, automaton);
					automaton.add(a, None, ea);
					a = eb;
				}

				let b = new_state();
				automaton.add(a, None, b);

				(a, b)
			}
			Self::Set(charset) => {
				let a = new_state();
				let b = new_state();

				automaton.add(a, Some(charset.clone()), b);
				(a, b)
			}
			Self::Union(exps) => {
				let a = new_state();
				let b = new_state();

				for e in exps {
					let (ea, eb) = e.build_into(new_state, automaton);
					automaton.add(a, None, ea);
					automaton.add(eb, None, b);
				}

				(a, b)
			}
		}
	}

	fn build_repeat_into(
		&self,
		new_state: &mut impl FnMut() -> usize,
		automaton: &mut Automaton<usize>,
		min: u32,
		max: u32,
	) -> (usize, usize) {
		if max == 0 {
			let a = new_state();
			(a, a)
		} else if min > 0 {
			let (a, b) = self.build_into(new_state, automaton);
			let (rest_a, rest_b) = self.build_repeat_into(
				new_state,
				automaton,
				min - 1,
				if max < u32::MAX { max - 1 } else { u32::MAX },
			);
			automaton.add(b, None, rest_a);
			(a, rest_b)
		} else if max < u32::MAX {
			let (a, b) = self.build_into(new_state, automaton);
			let (c, d) = self.build_repeat_into(new_state, automaton, 0, max - 1);
			automaton.add(a, None, d);
			automaton.add(b, None, c);
			(a, d)
		} else {
			let (a, b) = self.build_into(new_state, automaton);
			automaton.add(a, None, b);
			automaton.add(b, None, a);
			(a, b)
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	#[error("missing opening `(`")]
	UnmatchedClosingParenthesis,

	#[error("missing closing `)`")]
	MissingClosingParenthesis,

	#[error("incomplete escape sequence")]
	IncompleteEscapeSequence,

	#[error("incomplete character set")]
	IncompleteCharacterSet,
}

fn parse_charset(chars: &mut impl Iterator<Item = char>) -> Result<RangeSet<char>, ParseError> {
	#[derive(PartialEq, Eq)]
	enum State {
		Start,
		RangeStart,
		RangeDashOrStart,
		RangeEnd,
	}

	let mut state = State::Start;
	let mut negate = false;
	let mut set = RangeSet::new();

	let mut range_start = None;

	loop {
		match chars.next() {
			Some(c) => match c {
				'^' if state == State::Start => {
					negate = true;
					state = State::RangeStart;
				}
				c => match state {
					State::RangeDashOrStart if c == '-' => state = State::RangeEnd,
					State::Start | State::RangeStart | State::RangeDashOrStart if c == ']' => {
						if let Some(start) = range_start.take() {
							set.insert(start);
						}

						if negate {
							set = set.complement();
						}

						break Ok(set);
					}
					State::Start | State::RangeStart | State::RangeDashOrStart => {
						if let Some(start) = range_start.take() {
							set.insert(start);
						}

						let c = match c {
							'\\' => parse_escaped_char(chars)?,
							c => c,
						};

						range_start = Some(c);
						state = State::RangeDashOrStart
					}
					State::RangeEnd => {
						let c = match c {
							'\\' => parse_escaped_char(chars)?,
							c => c,
						};

						set.insert(range_start.take().unwrap()..=c);
						state = State::RangeStart
					}
				},
			},
			None => break Err(ParseError::IncompleteCharacterSet),
		}
	}
}

fn parse_escaped_char(chars: &mut impl Iterator<Item = char>) -> Result<char, ParseError> {
	match chars.next() {
		Some(c) => match c {
			'0' => Ok('\0'),
			'a' => Ok('\x07'),
			'b' => Ok('\x08'),
			't' => Ok('\t'),
			'n' => Ok('\n'),
			'v' => Ok('\x0b'),
			'f' => Ok('\x0c'),
			'r' => Ok('\r'),
			'e' => Ok('\x1b'),
			c => Ok(c),
		},
		None => Err(ParseError::IncompleteEscapeSequence),
	}
}

impl FromStr for RegExp {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s)
	}
}

impl<S: AsRef<str>> From<S> for RegExp {
	fn from(s: S) -> Self {
		let mut regexp = Self::empty();
		for c in s.as_ref().chars() {
			let mut charset = RangeSet::new();
			charset.insert(c);
			regexp.push(Self::Set(charset))
		}
		regexp
	}
}

const CHAR_COUNT: u64 = 0xd7ff + 0x10ffff - 0xe000;

impl fmt::Debug for RegExp {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, f)
	}
}

impl fmt::Display for RegExp {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Any => write!(f, "."),
			Self::Set(charset) => {
				if charset.len() == 1 {
					let c = charset.iter().next().unwrap().first().unwrap();
					fmt_char(c, f)
				} else {
					write!(f, "[")?;
					if charset.len() > CHAR_COUNT / 2 {
						write!(f, "^")?;
						for range in charset.gaps() {
							fmt_range(range.cloned(), f)?
						}
					} else {
						for range in charset {
							fmt_range(*range, f)?
						}
					}

					write!(f, "]")
				}
			}
			Self::Sequence(seq) => {
				for item in seq {
					if seq.len() > 1 {
						item.display_sub().fmt(f)?
					} else {
						item.fmt(f)?
					}
				}

				Ok(())
			}
			Self::Repeat(e, 0, 1) => write!(f, "{}?", e.display_sub()),
			Self::Repeat(e, 0, u32::MAX) => write!(f, "{}*", e.display_sub()),
			Self::Repeat(e, 1, u32::MAX) => write!(f, "{}+", e.display_sub()),
			Self::Repeat(e, min, u32::MAX) => write!(f, "{}{{{},}}", e.display_sub(), min),
			Self::Repeat(e, 0, max) => write!(f, "{}{{,{}}}", e.display_sub(), max),
			Self::Repeat(e, min, max) => {
				if min == max {
					write!(f, "{}{{{}}}", e.display_sub(), min)
				} else {
					write!(f, "{}{{{},{}}}", e.display_sub(), min, max)
				}
			}
			Self::Union(items) => {
				for (i, item) in items.iter().enumerate() {
					if i > 0 {
						write!(f, "|")?
					}

					item.display_sub().fmt(f)?
				}

				Ok(())
			}
		}
	}
}

/// Display the inner regular expression as a sub expression.
///
/// This will enclose it between parenthesis if necessary.
pub struct DisplaySub<'a>(&'a RegExp);

impl fmt::Display for DisplaySub<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.0.is_simple() {
			self.0.fmt(f)
		} else {
			write!(f, "({})", self.0)
		}
	}
}

fn fmt_range(range: btree_range_map::AnyRange<char>, f: &mut fmt::Formatter) -> fmt::Result {
	if range.len() == 1 {
		fmt_char(range.first().unwrap(), f)
	} else {
		let a = range.first().unwrap();
		let b = range.last().unwrap();

		fmt_char(a, f)?;
		if a as u32 + 1 < b as u32 {
			write!(f, "-")?;
		}
		fmt_char(b, f)
	}
}

fn fmt_char(c: char, f: &mut fmt::Formatter) -> fmt::Result {
	match c {
		'(' => write!(f, "\\("),
		')' => write!(f, "\\)"),
		'[' => write!(f, "\\["),
		']' => write!(f, "\\]"),
		'{' => write!(f, "\\{{"),
		'}' => write!(f, "\\}}"),
		'?' => write!(f, "\\?"),
		'*' => write!(f, "\\*"),
		'+' => write!(f, "\\+"),
		'-' => write!(f, "\\-"),
		'^' => write!(f, "\\^"),
		'|' => write!(f, "\\|"),
		'\\' => write!(f, "\\\\"),
		'\0' => write!(f, "\\0"),
		'\x07' => write!(f, "\\a"),
		'\x08' => write!(f, "\\b"),
		'\t' => write!(f, "\\t"),
		'\n' => write!(f, "\\n"),
		'\x0b' => write!(f, "\\v"),
		'\x0c' => write!(f, "\\f"),
		'\r' => write!(f, "\\r"),
		'\x1b' => write!(f, "\\e"),
		_ => fmt::Display::fmt(&c, f),
	}
}

impl serde::Serialize for RegExp {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.to_string().serialize(serializer)
	}
}

impl<'de> serde::Deserialize<'de> for RegExp {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl serde::de::Visitor<'_> for Visitor {
			type Value = RegExp;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				write!(formatter, "a regular expression")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				RegExp::parse(v).map_err(|e| E::custom(e))
			}
		}

		deserializer.deserialize_str(Visitor)
	}
}

#[cfg(test)]
mod tests {
	// Each pair is of the form `(regexp, formatted)`.
	// We check that the regexp is correctly parsed by formatting it and
	// checking that it matches the expected `formatted` string.
	const TESTS: &[(&str, &str)] = &[
		("a*", "a*"),
		("a\\*", "a\\*"),
		("[cab]", "[a-c]"),
		("[^cab]", "[^a-c]"),
		("(abc)|de", "abc|de"),
		("(a|b)?", "(a|b)?"),
		("[A-Za-z0-89]", "[0-9A-Za-z]"),
		("[a|b]", "[ab\\|]"),
	];

	#[test]
	fn test() {
		for (regexp, formatted) in TESTS {
			assert_eq!(
				super::RegExp::parse(regexp).unwrap().to_string(),
				*formatted
			)
		}
	}
}
