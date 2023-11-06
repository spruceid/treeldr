use std::collections::BTreeMap;

use grdf::Dataset;
use rdf_types::{Interpretation, ReverseIriInterpretation, ReverseLiteralInterpretation};
use treeldr::{Ref, Context, Layout, layout::{LiteralLayout, ListLayout, LayoutType}};

use crate::TypedValue;

pub enum Error {
	IncompatibleLayout,
	AbstractLayout,
	InvalidInputCount {
		expected: u32,
		found: u32
	}
}

/// Serialize the given RDF `dataset` using the provided `layout`, returning
/// a typed value.
pub fn hydrate<V, I: Interpretation, D>(
	vocabulary: &V,
	interpretation: &I,
	context: &Context<I::Resource>,
	dataset: &D,
	layout_ref: &Ref<I::Resource, LayoutType>,
	inputs: &[&I::Resource]
) -> Result<TypedValue<I::Resource>, Error>
where
	I: ReverseIriInterpretation + ReverseLiteralInterpretation,
	I::Resource: Clone,
	D: Dataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, GraphLabel = I::Resource>
{
	match context.get(layout_ref).unwrap() {
		Layout::Never => {
			Err(Error::IncompatibleLayout)
		},
		Layout::Literal(layout) => {
			if inputs.len() == 1 {
				let id = inputs[0];

				match layout {
					LiteralLayout::Data(_) => {
						for l in interpretation.literals_of(id) {
							// ...
						}

						todo!() // Error: no literal matching the layout
					}
					LiteralLayout::Id(_) => {
						for i in interpretation.iris_of(id) {
							// ...
						}

						todo!() // Error no IRI matching the layout
					}
				}
			} else {
				Err(Error::InvalidInputCount {
					expected: 1,
					found: inputs.len() as u32
				})
			}
		}
		Layout::Sum(layout) => {
			let tree = context.serialization_tree(layout_ref.id()).unwrap();

			// ...
			
			todo!()
		}
		Layout::Product(layout) => {
			let mut value = BTreeMap::new();
					
			for (name, field) in &layout.fields {
				// ...
			}

			Ok(TypedValue::Record(value, layout_ref.casted()))
		}
		Layout::List(layout) => {
			match layout {
				ListLayout::Unsized(_) => {
					todo!()
				}
				ListLayout::Sized(_) => {
					todo!()
				}
			}
		}
		Layout::Always => {
			Err(Error::AbstractLayout)
		}
	}
}