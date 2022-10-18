pub enum Metadata<M> {
	BuiltIn,
	Extern(M),
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

	fn merge_into_btree_map_entry<K: Ord>(self, entry: std::collections::btree_map::Entry<K, Self>) where Self: Sized {
		use std::collections::btree_map::Entry;
		match entry {
			Entry::Vacant(e) => {
				e.insert(self);
			},
			Entry::Occupied(mut e) => e.get_mut().merge_with(self)
		}
	}
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