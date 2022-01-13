/// Source span.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Span {
	/// Start byte (included).
	start: usize,

	/// End byte (excluded).
	end: usize
}

impl Span {
	pub fn new(start: usize, end: usize) -> Self {
		Self {
			start,
			end: std::cmp::max(start, end)
		}
	}

	pub fn start(&self) -> usize {
		self.start
	}

	pub fn end(&self) -> usize {
		self.end
	}

	/// Returns the byte length of the span.
	pub fn len(&self) -> usize {
		self.end - self.start
	}

	pub fn push(&mut self, c: char) {
		self.end += c.len_utf8()
	}

	pub fn clear(&mut self) {
		self.start = self.end
	}

	pub fn set_end(&mut self, end: usize) {
		self.end = std::cmp::max(self.start, end)
	}
}

impl From<usize> for Span {
	fn from(p: usize) -> Self {
		Self::new(p, p)
	}
}

impl From<Span> for std::ops::Range<usize> {
	fn from(span: Span) -> Self {
		span.start()..span.end()
	}
}