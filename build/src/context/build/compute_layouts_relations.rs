use std::collections::HashMap;

use derivative::Derivative;
use locspan::Meta;
use treeldr::Id;

use crate::{Context, SubLayout, ParentLayout};

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct LayoutRelations<M> {
	pub sub: Vec<SubLayout<M>>,
	pub parent: Vec<Meta<ParentLayout, M>>,
}

impl<M> Context<M> {
	/// Compute the `use` relation between all the layouts.
	///
	/// A layout is used by another layout if it is the layout of one of its
	/// fields.
	/// The purpose of this function is to declare to each layout how it it used
	/// using the `layout::Definition::add_use` method.
	pub fn compute_layouts_relations(&mut self) -> HashMap<Id, LayoutRelations<M>>
	where
		M: Clone,
	{
		let mut result: HashMap<Id, LayoutRelations<M>> = HashMap::new();

		for (id, node) in &self.nodes {
			if let Some(layout) = node.value().layout.as_ref() {
				let sub_layouts = layout.sub_layouts(self);

				for sub_layout in &sub_layouts {
					result
						.entry(*sub_layout.layout)
						.or_default()
						.parent
						.push(Meta::new(
							ParentLayout {
								layout: *id,
								connection: sub_layout.connection,
							},
							sub_layout.layout.metadata().clone(),
						))
				}

				result.entry(*id).or_default().sub = sub_layouts
			}
		}

		result
	}
}