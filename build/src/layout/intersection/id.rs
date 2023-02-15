use std::collections::{BTreeMap, VecDeque};

use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, Id, IriIndex};

use crate::Context;

#[derive(Debug, Clone)]
pub struct IdIntersection<M>(BTreeMap<Id, M>);

impl<M> Default for IdIntersection<M> {
	fn default() -> Self {
		Self::empty()
	}
}

impl<M> IdIntersection<M> {
	pub fn new(Meta(id, m): Meta<Id, M>) -> Self {
		let mut map = BTreeMap::new();
		map.insert(id, m);

		Self(map)
	}

	pub fn empty() -> Self {
		Self(BTreeMap::new())
	}

	pub fn iter(&self) -> impl '_ + Iterator<Item = Meta<Id, &M>> {
		self.0.iter().map(|(id, m)| Meta(*id, m))
	}
}

impl<M: Merge> IdIntersection<M> {
	pub fn insert(&mut self, Meta(value, meta): Meta<Id, M>) {
		use std::collections::btree_map::Entry;
		match self.0.entry(value) {
			Entry::Occupied(mut entry) => entry.get_mut().merge_with(meta),
			Entry::Vacant(entry) => {
				entry.insert(meta);
			}
		}
	}

	pub fn intersection(&self, other: &Self) -> Self
	where
		M: Clone + Merge,
	{
		let mut result = self.clone();

		for v in other.iter() {
			result.insert(v.cloned_metadata())
		}

		result
	}

	pub fn intersect_with(&mut self, other: Self)
	where
		M: Clone + Merge,
	{
		for (v, m) in other.0 {
			self.insert(Meta(v, m))
		}
	}

	pub fn intersected_with(mut self, other: Self) -> Self
	where
		M: Clone + Merge,
	{
		self.intersect_with(other);
		self
	}

	pub fn prepare_layout<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut VecDeque<Id>,
		meta: M,
	) -> Id
	where
		M: Clone,
	{
		self.0
			.keys()
			.find(|a| {
				self.0
					.keys()
					.all(|b| crate::layout::is_included_in(context, **a, *b))
			})
			.copied()
			.unwrap_or_else(|| {
				let id = generator.next(vocabulary);

				let list_id = context.create_list_with(
					vocabulary,
					generator,
					self.0,
					|(layout_id, layout_meta), _, _, _| Meta(layout_id.into_term(), layout_meta),
				);

				let node = context.declare_layout(id, meta.clone());
				node.as_layout_mut()
					.intersection_of_mut()
					.insert_base(Meta(list_id, meta));

				stack.push_back(id);
				id
			})
	}
}

impl<M, N> PartialEq<IdIntersection<N>> for IdIntersection<M> {
	fn eq(&self, other: &IdIntersection<N>) -> bool {
		self.0.keys().eq(other.0.keys())
	}
}

impl<M> Eq for IdIntersection<M> {}

impl<M, N> PartialOrd<IdIntersection<N>> for IdIntersection<M> {
	fn partial_cmp(&self, other: &IdIntersection<N>) -> Option<std::cmp::Ordering> {
		self.0.keys().partial_cmp(other.0.keys())
	}
}

impl<M> Ord for IdIntersection<M> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.0.keys().cmp(other.0.keys())
	}
}
