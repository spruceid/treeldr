use std::collections::VecDeque;

use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, IriIndex};

use crate::{component, Context, Error};

impl<M> Context<M> {
	pub(crate) fn compute_layout_intersections<
		V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		let mut stack = VecDeque::new();

		for (id, node) in &self.nodes {
			let is_layout = node.has_type(self, component::Type::Layout);
			if is_layout && !node.as_layout().intersection_of().is_empty() {
				stack.push_back(*id)
			}
		}

		while let Some(id) = stack.pop_front() {
			let node = self.get(id).unwrap();

			match node
				.as_layout()
				.intersection_definition(self, node.as_resource())?
			{
				Some(def) => {
					let def = def.build(vocabulary, generator, self, &mut stack)?;

					let node = self.get_mut(id).unwrap();
					node.as_layout_mut().add(def)
				}
				None => stack.push_back(id),
			}
		}

		Ok(())
	}
}
