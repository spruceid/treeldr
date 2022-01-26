/// Three peekable iterator.
pub struct Peekable3<I: Iterator> {
	next1: Option<Option<I::Item>>,
	next2: Option<Option<I::Item>>,
	next3: Option<Option<I::Item>>,
	inner: I
}

impl<I: Iterator> Peekable3<I> {
	pub fn new(iter: I) -> Self {
		Self {
			next1: None,
			next2: None,
			next3: None,
			inner: iter
		}
	}

	pub fn peek(&mut self) -> Option<&I::Item> {
		if self.next1.is_none() {
			let item = self.inner.next();
			self.next1 = Some(item);
		}

		self.next1.as_ref().unwrap().as_ref()
	}

	pub fn peek2(&mut self) -> Option<&I::Item> {
		if self.next1.is_none() {
			let item = self.inner.next();
			self.next1 = Some(item);
		}

		if self.next2.is_none() {
			let item = self.inner.next();
			self.next2 = Some(item);
		}

		self.next2.as_ref().unwrap().as_ref()
	}

	pub fn peek3(&mut self) -> Option<&I::Item> {
		if self.next1.is_none() {
			let item = self.inner.next();
			self.next1 = Some(item);
		}

		if self.next2.is_none() {
			let item = self.inner.next();
			self.next2 = Some(item);
		}

		if self.next3.is_none() {
			let item = self.inner.next();
			self.next3 = Some(item);
		}

		self.next3.as_ref().unwrap().as_ref()
	}
}

impl<I: Iterator> Iterator for Peekable3<I> {
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		match self.next1.take() {
			Some(item) => {
				self.next1 = self.next2.take();
				self.next2 = self.next3.take();
				item
			},
			None => {
				self.inner.next()
			}
		}
	}
}