use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Default, Clone)]
pub struct Path(Vec<Segment>);

impl Path {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn segments(&self) -> std::slice::Iter<Segment> {
		self.0.iter()
	}

	pub fn longest_common_prefix(&self, other: &Self) -> Self {
		self.segments()
			.zip(other.segments())
			.take_while(|(a, b)| a == b)
			.map(|(s, _)| s)
			.cloned()
			.collect()
	}

	pub fn to(&self, other: &Self) -> Self {
		let lcp = self.longest_common_prefix(other);
		let mut path = Path::new();

		for _ in lcp.len()..self.len() {
			path.push(Segment::Super)
		}

		for i in lcp.len()..other.len() {
			path.push(other.0[i].clone())
		}

		path
	}

	pub fn push(&mut self, segment: impl Into<Segment>) {
		self.0.push(segment.into())
	}
}

impl FromIterator<Segment> for Path {
	fn from_iter<I: IntoIterator<Item = Segment>>(iter: I) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl ToTokens for Path {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		for (i, segment) in self.segments().enumerate() {
			if i > 0 {
				tokens.extend(quote! { :: })
			}

			segment.to_tokens(tokens)
		}
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Segment {
	Super,
	Ident(proc_macro2::Ident),
}

impl From<proc_macro2::Ident> for Segment {
	fn from(value: proc_macro2::Ident) -> Self {
		Self::Ident(value)
	}
}

impl quote::ToTokens for Segment {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Super => tokens.extend(quote! { super }),
			Self::Ident(id) => id.to_tokens(tokens),
		}
	}
}
