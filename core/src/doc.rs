#[derive(Default, Clone)]
pub struct Documentation {
	short: Option<String>,
	long: Option<String>,
}

impl Documentation {
	pub fn new(short: Option<String>, long: Option<String>) -> Self {
		Self { short, long }
	}

	pub fn is_empty(&self) -> bool {
		self.short.is_none() && self.long.is_none()
	}

	pub fn short_description(&self) -> Option<&str> {
		self.short.as_deref()
	}

	pub fn long_description(&self) -> Option<&str> {
		self.long.as_deref()
	}
}
