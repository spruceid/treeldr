use std::{collections::{BTreeMap, HashMap}, marker::PhantomData};

use rdf_types::{Interpretation, ReverseIriInterpretation, ReverseLiteralInterpretation};
use treeldr::{Ref, Context, Layout, layout::{LiteralLayout, ListLayout, LayoutType}, graph::Dataset, Pattern};

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
	inputs: &[I::Resource]
) -> Result<TypedValue<I::Resource>, Error>
where
	I: ReverseIriInterpretation + ReverseLiteralInterpretation,
	I::Resource: Clone,
	D: grdf::Dataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, GraphLabel = I::Resource>
{
	match context.get(layout_ref).unwrap() {
		Layout::Never => {
			Err(Error::IncompatibleLayout)
		},
		Layout::Literal(layout) => {
			if inputs.len() == 1 {
				let id = &inputs[0];

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
			let discriminants = context.serialization_discriminants(layout_ref.id()).unwrap();

			// ...
			
			todo!()
		}
		Layout::Product(layout) => {
			let mut value = BTreeMap::new();
					
			for field in &layout.fields {
				// ...
			}

			Ok(TypedValue::Record(value, layout_ref.casted()))
		}
		Layout::List(layout) => {
			match layout {
				ListLayout::Unsized(layout) => {
					if layout.input == inputs.len() as u32 {
						let mut substitution = Substitution::from_inputs(inputs);
						substitution.intro(layout.intro);
						
						match &layout.sequence {
							Some(s) => {
								// let mut substitution = substitution.clone();
								// substitution.intro(layout.item);
								// let matching = Matching::new(
								// 	substitution,
								// 	layout.graph.iter().chain(layout.item.graph)
								// );

								todo!()
							}
							None => {
								todo!()
							}
						}
					} else {
						Err(Error::InvalidInputCount {
							expected: layout.input,
							found: inputs.len() as u32
						})
					}
				}
				ListLayout::Sized(layout) => {
					if layout.input == inputs.len() as u32 {
						todo!()
					} else {
						Err(Error::InvalidInputCount {
							expected: layout.input,
							found: inputs.len() as u32
						})
					}
				}
			}
		}
		Layout::Always => {
			Err(Error::AbstractLayout)
		}
	}
}

#[derive(Clone)]
pub struct Substitution<R>(Vec<Option<R>>);

impl<R> Substitution<R> {
	pub fn new() -> Self {
		Self(Vec::new())
	}

	pub fn from_inputs(inputs: &[R]) -> Self where R: Clone {
		Self(inputs.iter().cloned().map(Some).collect())
	}

	pub fn len(&self) -> u32 {
		self.0.len() as u32
	}

	pub fn intro(&mut self, count: u32) {
		self.0.resize_with(self.0.len() + count as usize, || None)
	}
}

impl<R> Default for Substitution<R> {
	fn default() -> Self {
		Self::new()
	}
}

pub struct Matching<R>(PhantomData<R>);

impl<R> Matching<R> {
	pub fn new() -> Self {
		todo!()
	}
}

impl<R> Iterator for Matching<R> {
	type Item = Substitution<R>;

	fn next(&mut self) -> Option<Self::Item> {
		todo!()
	}
}