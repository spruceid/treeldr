use btree_range_map::RangeSet;
use std::fmt;

/// Regular expression.
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
	Repeat(Box<Self>, usize, usize),

	/// Union.
	Union(Vec<Self>),
}

impl RegExp {
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
}

const CHAR_COUNT: u64 = 0xd7ff + 0x10ffff - 0xe000;

impl fmt::Display for RegExp {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Any => write!(f, "."),
			Self::Set(charset) => {
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

				Ok(())
			}
			Self::Sequence(seq) => {
				for item in seq {
					item.fmt(f)?
				}

				Ok(())
			}
			Self::Repeat(e, 0, 1) => write!(f, "{}?", e.display_sub()),
			Self::Repeat(e, 0, usize::MAX) => write!(f, "{}*", e.display_sub()),
			Self::Repeat(e, 1, usize::MAX) => write!(f, "{}+", e.display_sub()),
			Self::Repeat(e, min, usize::MAX) => write!(f, "{}{{{},}}", e.display_sub(), min),
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

impl<'a> fmt::Display for DisplaySub<'a> {
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
		fmt_char(range.first().unwrap(), f)?;
		write!(f, "-")?;
		fmt_char(range.last().unwrap(), f)
	}
}

fn fmt_char(c: char, f: &mut fmt::Formatter) -> fmt::Result {
	match c {
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
