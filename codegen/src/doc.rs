use std::fmt;
use crate::IndentBy;

pub enum CommentSyntax {
	Single(&'static str),
	Multi {
		start: &'static str,
		middle: &'static str,
		end: &'static str
	}
}

pub enum CommentPosition {
	Start,
	Middle,
	End
}

impl CommentSyntax {
	pub fn as_str(&self, pos: CommentPosition) -> &'static str {
		match self {
			Self::Single(s) => s,
			Self::Multi { start, middle, end } => match pos {
				CommentPosition::Start => start,
				CommentPosition::Middle => middle,
				CommentPosition::End => end
			}
		}
	}
}

pub struct Comment<S> {
	content: S,
	syntax: CommentSyntax,
	indent_by: IndentBy
}

impl<S> Comment<S> {
	pub fn new(content: S, syntax: CommentSyntax, indent_by: IndentBy) -> Self {
		Self {
			content,
			syntax,
			indent_by
		}
	}
}

impl<S: AsRef<str>> fmt::Display for Comment<S> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let content = self.content.as_ref();
		let line_count = content.lines().count();

		for (i, line) in content.lines().enumerate() {
			let pos = if i == 0 {
				CommentPosition::Start
			} else if i+1 == line_count {
				CommentPosition::End
			} else {
				CommentPosition::Middle
			};

			if i > 0 {
				writeln!(f, "")?;
			}

			write!(f, "{}{}{}", self.indent_by, self.syntax.as_str(pos), line)?;

			if i == 0 && i+1 == line_count {
				write!(f, "{}", self.syntax.as_str(CommentPosition::End))?;
			}
		}

		Ok(())
	}
}