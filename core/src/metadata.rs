use locspan::MaybeLocated;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Metadata<M> {
	BuiltIn,
	Extern(M),
}

impl<M> Default for Metadata<M> {
	fn default() -> Self {
		Self::BuiltIn
	}
}

impl<M: MaybeLocated> MaybeLocated for Metadata<M> {
	type File = M::File;
	type Span = M::Span;

	fn optional_location(&self) -> Option<&locspan::Location<Self::File, Self::Span>> {
		match self {
			Self::BuiltIn => None,
			Self::Extern(m) => m.optional_location(),
		}
	}
}

pub trait Merge {
	fn merge_with(&mut self, other: Self);

	fn merged_with(mut self, other: Self) -> Self
	where
		Self: Sized,
	{
		self.merge_with(other);
		self
	}

	fn merge_into_btree_map_entry<K: Ord>(self, entry: std::collections::btree_map::Entry<K, Self>)
	where
		Self: Sized,
	{
		use std::collections::btree_map::Entry;
		match entry {
			Entry::Vacant(e) => {
				e.insert(self);
			}
			Entry::Occupied(mut e) => e.get_mut().merge_with(self),
		}
	}
}

impl Merge for () {
	fn merge_with(&mut self, _other: Self) {}
}

impl Merge for locspan::Span {
	fn merge_with(&mut self, other: Self) {
		*self = other
	}
}

impl<F, S> Merge for locspan::Location<F, S> {
	fn merge_with(&mut self, other: Self) {
		*self = other
	}
}

impl<M: Merge> Merge for Metadata<M> {
	fn merge_with(&mut self, other: Self) {
		match (self, other) {
			(Self::BuiltIn, _) => (),
			(this, Self::BuiltIn) => *this = Self::BuiltIn,
			(Self::Extern(a), Self::Extern(b)) => a.merge_with(b),
		}
	}
}
