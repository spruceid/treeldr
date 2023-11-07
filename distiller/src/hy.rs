use std::{
	collections::{BTreeMap, HashMap},
	marker::PhantomData,
};

use rdf_types::{Interpretation, Quad, ReverseIriInterpretation, ReverseLiteralInterpretation};
use treeldr::{
	graph::Dataset,
	layout::{LayoutType, ListLayout, LiteralLayout},
	pattern::Substitution,
	Context, Layout, Pattern, Ref,
};

use crate::TypedValue;

pub enum Error {
	IncompatibleLayout,
	AbstractLayout,
	InvalidInputCount { expected: u32, found: u32 },
}

/// Serialize the given RDF `dataset` using the provided `layout`, returning
/// a typed value.
pub fn hydrate<V, I: Interpretation, D>(
	vocabulary: &V,
	interpretation: &I,
	context: &Context<I::Resource>,
	dataset: &D,
	current_graph: Option<&I::Resource>,
	layout_ref: &Ref<I::Resource, LayoutType>,
	inputs: &[I::Resource],
) -> Result<TypedValue<I::Resource>, Error>
where
	I: ReverseIriInterpretation + ReverseLiteralInterpretation,
	I::Resource: Clone + PartialEq,
	D: grdf::Dataset<
		Subject = I::Resource,
		Predicate = I::Resource,
		Object = I::Resource,
		GraphLabel = I::Resource,
	>,
{
	match context.get(layout_ref).unwrap() {
		Layout::Never => Err(Error::IncompatibleLayout),
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
					found: inputs.len() as u32,
				})
			}
		}
		Layout::Sum(layout) => {
			let discriminants = context
				.serialization_discriminants(layout_ref.id())
				.unwrap();

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
				ListLayout::Unordered(layout) => {
					if layout.input == inputs.len() as u32 {
						let mut substitution = Substitution::from_inputs(inputs);
						substitution.intro(layout.intro);

						let mut matching = Matching::new(
							substitution,
							layout.dataset.quads().chain(&layout.item.dataset),
						);

						for m in matching {
							// TODO build item
						}

						todo!()
					} else {
						Err(Error::InvalidInputCount {
							expected: layout.input,
							found: inputs.len() as u32,
						})
					}
				}
				ListLayout::Ordered(layout) => {
					if layout.input == inputs.len() as u32 {
						let mut substitution = Substitution::from_inputs(inputs);
						substitution.intro(layout.intro);

						let mut head = layout.head.apply(&substitution).into_resource().unwrap();
						let tail = layout.tail.apply(&substitution).into_resource().unwrap();

						let mut items = Vec::new();

						while head != tail {
							let mut item_substitution = substitution.clone();
							item_substitution.push(Some(head)); // the head
							let rest = item_substitution.intro(1 + layout.node.intro); // the rest, and other intro variables.

							let mut matching = Matching::new(
								item_substitution,
								layout
									.dataset
									.quads()
									.chain(&layout.node.dataset)
									.with_graph(current_graph),
							);

							match matching.next() {
								Some(item_substitution) => {
									match matching.next() {
										Some(_) => {
											todo!() // Error: ambiguity
										}
										None => {
											constrain_outer_substitution(
												&mut substitution,
												&item_substitution,
												layout.input,
												layout.intro,
											);

											// let rest = item_substitution.get(rest).unwrap().clone();
											let item_inputs: Vec<_> = layout
												.node
												.format
												.inputs
												.iter()
												.map(|p| {
													p.apply(&item_substitution)
														.into_resource()
														.unwrap()
												})
												.collect();

											let item_graph = select_graph(
												current_graph,
												&layout.node.format.graph,
												&item_substitution,
											);

											let item = hydrate(
												vocabulary,
												interpretation,
												context,
												dataset,
												item_graph.as_ref(),
												&layout.node.format.layout,
												&item_inputs,
											)?;

											items.push(item);

											head = item_substitution.get(rest).unwrap().clone();
										}
									}
								}
								None => {
									todo!() // Error: missing item.
								}
							}
						}

						Ok(TypedValue::List(items, layout_ref.casted()))
					} else {
						Err(Error::InvalidInputCount {
							expected: layout.input,
							found: inputs.len() as u32,
						})
					}
				}
				ListLayout::Sized(layout) => {
					if layout.input == inputs.len() as u32 {
						let mut substitution = Substitution::from_inputs(inputs);
						substitution.intro(layout.intro);

						let mut matching = Matching::new(
							substitution,
							layout.dataset.quads().with_graph(current_graph),
						);

						match matching.next() {
							Some(m) => {
								match matching.next() {
									Some(_) => {
										todo!() // Error: ambiguity
									}
									None => {
										let mut items = Vec::with_capacity(layout.formats.len());

										for item_format in &layout.formats {
											let item_inputs: Vec<_> = item_format
												.inputs
												.iter()
												.map(|p| p.apply(&m).into_resource().unwrap())
												.collect();

											let item_graph =
												select_graph(current_graph, &item_format.graph, &m);

											let item = hydrate(
												vocabulary,
												interpretation,
												context,
												dataset,
												item_graph.as_ref(),
												&item_format.layout,
												&item_inputs,
											)?;

											items.push(item)
										}

										Ok(TypedValue::List(items, layout_ref.casted()))
									}
								}
							}
							None => {
								todo!() // Error: not found
							}
						}
					} else {
						Err(Error::InvalidInputCount {
							expected: layout.input,
							found: inputs.len() as u32,
						})
					}
				}
			}
		}
		Layout::Always => Err(Error::AbstractLayout),
	}
}

fn constrain_outer_substitution<R: Clone>(
	outer: &mut Substitution<R>,
	inner: &Substitution<R>,
	offset: u32,
	len: u32,
) {
	for x in offset..(offset + len) {
		outer.set(x, inner.get(x).cloned())
	}
}

fn select_graph<R: Clone>(
	current_graph: Option<&R>,
	graph_pattern: &Option<Option<Pattern<R>>>,
	substitution: &Substitution<R>,
) -> Option<R> {
	graph_pattern
		.as_ref()
		.map(|g| {
			g.as_ref()
				.map(|p| p.apply(substitution).into_resource().unwrap())
		})
		.unwrap_or_else(|| current_graph.cloned())
}

pub trait QuadsWithGraphExt<'a, R>: Sized {
	fn with_graph(self, graph: Option<&'a R>) -> QuadsWithGraph<'a, R, Self>;
}

impl<'a, R: 'a, I> QuadsWithGraphExt<'a, R> for I
where
	I: Iterator<Item = Quad<&'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>>>,
{
	fn with_graph(self, graph: Option<&'a R>) -> QuadsWithGraph<'a, R, Self> {
		QuadsWithGraph { quads: self, graph }
	}
}

pub struct QuadsWithGraph<'a, R, I> {
	quads: I,
	graph: Option<&'a R>,
}

impl<'a, R, I> Iterator for QuadsWithGraph<'a, R, I>
where
	I: Iterator<Item = Quad<&'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>>>,
{
	type Item = Quad<Pattern<&'a R>, Pattern<&'a R>, Pattern<&'a R>, Pattern<&'a R>>;

	fn next(&mut self) -> Option<Self::Item> {
		todo!()
	}
}

pub struct Matching<R, D> {
	quads: D,
	r: PhantomData<R>,
}

impl<R, D> Matching<R, D> {
	pub fn new(substitution: Substitution<R>, quads: D) -> Self {
		todo!()
	}
}

impl<R, D> Iterator for Matching<R, D> {
	type Item = Substitution<R>;

	fn next(&mut self) -> Option<Self::Item> {
		todo!()
	}
}
