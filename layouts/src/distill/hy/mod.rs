use std::collections::BTreeMap;

use crate::{
	abs::syntax::{OrderedListLayoutType, SizedListLayoutType},
	layout::{LayoutType, ListLayout, LiteralLayout, ProductLayoutType, SumLayoutType},
	matching,
	pattern::Substitution,
	utils::QuadsExt,
	Layout, LayoutRegistry, Matching, Pattern, Ref, TypedLiteral, TypedValue, Value,
};
use iref::IriBuf;
use rdf_types::{
	dataset::{PatternMatchingDataset, TraversableDataset},
	interpretation::{ReverseIriInterpretation, ReverseLiteralInterpretation},
	Interpretation, Term, Vocabulary,
};

mod data;

use data::*;

/// Hydrate error.
#[derive(Debug, thiserror::Error)]
pub enum Error<R = Term> {
	#[error("incompatible layout")]
	IncompatibleLayout,

	#[error("invalid input count (expected {expected}, found {found})")]
	InvalidInputCount { expected: u32, found: u32 },

	#[error("ambiguous {0}")]
	DataAmbiguity(Box<DataFragment<R>>),

	#[error("missing required {0}")]
	MissingData(Box<DataFragment<R>>),

	#[error("unknown number datatype")]
	UnknownNumberDatatype(IriBuf),

	#[error("no matching literal representation found")]
	NoMatchingLiteral,

	#[error("layout `{0}` is undefined")]
	LayoutNotFound(Ref<LayoutType, R>),
}

static_assertions::assert_impl_all!(Error: ToString);

#[derive(Debug, thiserror::Error)]
pub enum DataFragment<R> {
	#[error("layout discriminant")]
	Discriminant(Ref<LayoutType, R>),

	#[error("variant `{variant_name}`")]
	Variant {
		layout: Ref<SumLayoutType, R>,
		variant_name: String,
	},

	#[error("key `{key}`")]
	Key {
		layout: Ref<ProductLayoutType, R>,
		key: Value,
	},

	#[error("list node")]
	OrderedListNode {
		layout: Ref<OrderedListLayoutType, R>,
		head: R,
		tail: R,
	},

	#[error("list item")]
	SizedListItem {
		layout: Ref<SizedListLayoutType, R>,
		index: usize,
	},
}

impl<R> Error<R> {
	fn from_matching_error(value: matching::Error, f: DataFragment<R>) -> Self {
		match value {
			matching::Error::Ambiguity => Self::DataAmbiguity(Box::new(f)),
			matching::Error::Empty => Self::MissingData(Box::new(f)),
		}
	}
}

trait MatchingForFragment<R> {
	type Ok;

	fn for_fragment(self, f: impl FnOnce() -> DataFragment<R>) -> Result<Self::Ok, Error<R>>;
}

impl<T, R> MatchingForFragment<R> for Result<T, matching::Error> {
	type Ok = T;

	fn for_fragment(self, f: impl FnOnce() -> DataFragment<R>) -> Result<T, Error<R>> {
		self.map_err(|e| Error::from_matching_error(e, f()))
	}
}

/// Serialize the given RDF `dataset` using the provided `layout`.
///
/// This is a simplified version of [`hydrate_with`] using the basic unit `()`
/// interpretation where resources are interpreted as their lexical
/// representation (a [`Term`]).
///
/// The data to be serialized is contained in the given RDF `dataset`.
/// Serialization is performed following the layout identified by `layout_ref`
/// in the layout collection `layouts`. This layout requires a number of inputs
/// (the entry point to the dataset) provided by the `inputs` slice.
///
/// This function a tree value annotated (typed) with references to the
/// different layouts used to serialize each part of the tree.
pub fn hydrate<D>(
	context: impl LayoutRegistry,
	dataset: &D,
	layout_ref: &Ref<LayoutType>,
	inputs: &[Term],
) -> Result<TypedValue, Error>
where
	D: PatternMatchingDataset<Resource = Term>,
{
	hydrate_with(&(), &(), context, dataset, None, layout_ref, inputs)
}

/// Serialize the given RDF `dataset` using the provided `layout`, returning
/// a typed value.
pub fn hydrate_with<V, I: Interpretation, D>(
	vocabulary: &V,
	interpretation: &I,
	context: impl LayoutRegistry<I::Resource>,
	dataset: &D,
	current_graph: Option<&I::Resource>,
	layout_ref: &Ref<LayoutType, I::Resource>,
	inputs: &[I::Resource],
) -> Result<TypedValue<I::Resource>, Error<I::Resource>>
where
	V: Vocabulary,
	V::Iri: PartialEq,
	I: ReverseIriInterpretation<Iri = V::Iri> + ReverseLiteralInterpretation<Literal = V::Literal>,
	I::Resource: Clone + Ord,
	D: PatternMatchingDataset<Resource = I::Resource>,
{
	hydrate_with_ref(
		vocabulary,
		interpretation,
		&context,
		dataset,
		current_graph,
		layout_ref,
		inputs,
	)
}

fn hydrate_with_ref<V, I: Interpretation, D>(
	vocabulary: &V,
	interpretation: &I,
	context: &impl LayoutRegistry<I::Resource>,
	dataset: &D,
	current_graph: Option<&I::Resource>,
	layout_ref: &Ref<LayoutType, I::Resource>,
	inputs: &[I::Resource],
) -> Result<TypedValue<I::Resource>, Error<I::Resource>>
where
	V: Vocabulary,
	V::Iri: PartialEq,
	I: ReverseIriInterpretation<Iri = V::Iri> + ReverseLiteralInterpretation<Literal = V::Literal>,
	I::Resource: Clone + Ord,
	D: PatternMatchingDataset<Resource = I::Resource>,
{
	let layout = context
		.get(layout_ref)
		.ok_or_else(|| Error::LayoutNotFound(layout_ref.clone()))?;

	if let Some(expected) = layout.input_count().filter(|&i| i != inputs.len() as u32) {
		return Err(Error::InvalidInputCount {
			expected,
			found: inputs.len() as u32,
		});
	}

	match layout {
		Layout::Never => Err(Error::IncompatibleLayout),
		Layout::Literal(layout) => {
			match layout {
				LiteralLayout::Data(layout) => {
					let value = hydrate_data(
						vocabulary,
						interpretation,
						dataset,
						current_graph,
						layout_ref.casted(),
						layout,
						inputs,
					)?;

					Ok(TypedValue::Literal(value))
				}
				LiteralLayout::Id(layout) => {
					let mut substitution = Substitution::from_inputs(inputs);
					substitution.intro(layout.intro);
					let substitution = Matching::new(
						dataset,
						substitution.clone(),
						layout.dataset.quads().with_default_graph(current_graph),
					)
					.into_required_unique()
					.for_fragment(|| DataFragment::Discriminant(layout_ref.clone()))?;

					let resource = layout
						.resource
						.apply(&substitution)
						.into_resource()
						.unwrap();

					let mut selected = None;

					for i in interpretation.iris_of(&resource) {
						let iri = vocabulary.iri(i).unwrap();

						// TODO check automaton.

						if selected.replace(iri).is_some() {
							todo!() // Error: IRI ambiguity
						}
					}

					match selected {
						Some(iri) => Ok(TypedValue::Literal(TypedLiteral::Id(
							iri.to_string(),
							layout_ref.casted(),
						))),
						None => {
							todo!() // Error no IRI matching the layout
						}
					}
				}
			}
		}
		Layout::Sum(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);
			let substitution = Matching::new(
				dataset,
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph),
			)
			.into_required_unique()
			.for_fragment(|| DataFragment::Discriminant(layout_ref.clone()))?;

			let mut failures = Vec::new();
			let mut selected = None;

			for (i, variant) in layout.variants.iter().enumerate() {
				let mut variant_substitution = substitution.clone();
				variant_substitution.intro(variant.intro);

				let variant_substitution = Matching::new(
					dataset,
					substitution.clone(),
					variant.dataset.quads().with_default_graph(current_graph),
				)
				.into_unique()
				.for_fragment(|| DataFragment::Variant {
					layout: layout_ref.clone().cast(),
					variant_name: variant.name.clone(),
				})?;

				match variant_substitution {
					Some(variant_substitution) => {
						let variant_inputs =
							select_inputs(&variant.value.input, &variant_substitution);

						let variant_graph = select_graph(
							current_graph,
							&variant.value.graph,
							&variant_substitution,
						);

						let value = hydrate_with_ref(
							vocabulary,
							interpretation,
							context,
							dataset,
							variant_graph.as_ref(),
							&variant.value.layout,
							&variant_inputs,
						);

						match value {
							Ok(value) => {
								match selected.take() {
									Some((_j, _other_value)) => {
										todo!() // Error: variant ambiguity
									}
									None => selected = Some((i, value)),
								}
							}
							Err(e) => failures.push(Some(e)),
						}
					}
					None => failures.push(None),
				}
			}

			match selected {
				Some((i, value)) => Ok(TypedValue::Variant(
					Box::new(value),
					layout_ref.casted(),
					i as u32,
				)),
				None => {
					todo!() // Error: no variant found
				}
			}
		}
		Layout::Product(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);

			let substitution = Matching::new(
				dataset,
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph),
			)
			.into_required_unique()
			.for_fragment(|| DataFragment::Discriminant(layout_ref.clone()))?;

			let mut record = BTreeMap::new();

			for (key, field) in &layout.fields {
				let mut field_substitution = substitution.clone();
				field_substitution.intro(field.intro);

				let field_substitution = Matching::new(
					dataset,
					field_substitution,
					field.dataset.quads().with_default_graph(current_graph),
				)
				.into_unique()
				.for_fragment(|| DataFragment::Key {
					layout: layout_ref.clone().cast(),
					key: key.clone(),
				})?;

				match field_substitution {
					Some(field_substitution) => {
						let field_inputs = select_inputs(&field.value.input, &field_substitution);

						let item_graph =
							select_graph(current_graph, &field.value.graph, &field_substitution);

						let value = hydrate_with_ref(
							vocabulary,
							interpretation,
							context,
							dataset,
							item_graph.as_ref(),
							&field.value.layout,
							&field_inputs,
						)?;

						record.insert(key.clone(), value);
					}
					None => {
						if field.required {
							return Err(Error::MissingData(Box::new(DataFragment::Key {
								layout: layout_ref.clone().cast(),
								key: key.clone(),
							})));
						}
					}
				}
			}

			Ok(TypedValue::Map(record, layout_ref.casted()))
		}
		Layout::List(layout) => {
			match layout {
				ListLayout::Unordered(layout) => {
					let mut substitution = Substitution::from_inputs(inputs);
					substitution.intro(layout.intro);

					let mut item_substitution = Matching::new(
						dataset,
						substitution,
						layout.dataset.quads().with_default_graph(current_graph),
					)
					.into_required_unique()
					.for_fragment(|| DataFragment::Discriminant(layout_ref.clone()))?;

					item_substitution.intro(layout.item.intro);
					let matching = Matching::new(
						dataset,
						item_substitution,
						layout
							.item
							.dataset
							.quads()
							.with_default_graph(current_graph),
					);

					let mut items = Vec::new();

					for item_substitution in matching {
						let item_inputs =
							select_inputs(&layout.item.value.input, &item_substitution);

						let item_graph = select_graph(
							current_graph,
							&layout.item.value.graph,
							&item_substitution,
						);

						let item = hydrate_with_ref(
							vocabulary,
							interpretation,
							context,
							dataset,
							item_graph.as_ref(),
							&layout.item.value.layout,
							&item_inputs,
						)?;

						items.push(item);
					}

					items.sort_unstable();

					Ok(TypedValue::List(items, layout_ref.casted()))
				}
				ListLayout::Ordered(layout) => {
					let mut substitution = Substitution::from_inputs(inputs);
					substitution.intro(layout.intro);

					let substitution = Matching::new(
						dataset,
						substitution,
						layout.dataset.quads().with_default_graph(current_graph),
					)
					.into_required_unique()
					.for_fragment(|| DataFragment::Discriminant(layout_ref.clone()))?;

					let mut head = layout.head.apply(&substitution).into_resource().unwrap();
					let tail = layout.tail.apply(&substitution).into_resource().unwrap();

					let mut items = Vec::new();

					while head != tail {
						let mut item_substitution = substitution.clone();
						item_substitution.push(Some(head.clone())); // the head
						let rest = item_substitution.intro(1 + layout.node.intro); // the rest, and other intro variables.

						let item_substitution = Matching::new(
							dataset,
							item_substitution,
							layout
								.node
								.dataset
								.quads()
								.with_default_graph(current_graph),
						)
						.into_required_unique()
						.for_fragment(|| DataFragment::OrderedListNode {
							layout: layout_ref.clone().cast(),
							head,
							tail: tail.clone(),
						})?;

						let item_inputs =
							select_inputs(&layout.node.value.input, &item_substitution);

						let item_graph = select_graph(
							current_graph,
							&layout.node.value.graph,
							&item_substitution,
						);

						let item = hydrate_with_ref(
							vocabulary,
							interpretation,
							context,
							dataset,
							item_graph.as_ref(),
							&layout.node.value.layout,
							&item_inputs,
						)?;

						items.push(item);

						head = item_substitution.get(rest).unwrap().clone();
					}

					Ok(TypedValue::List(items, layout_ref.casted()))
				}
				ListLayout::Sized(layout) => {
					let mut substitution = Substitution::from_inputs(inputs);
					substitution.intro(layout.intro);

					let substitution = Matching::new(
						dataset,
						substitution,
						layout.dataset.quads().with_default_graph(current_graph),
					)
					.into_required_unique()
					.for_fragment(|| DataFragment::Discriminant(layout_ref.clone()))?;

					let mut items = Vec::with_capacity(layout.items.len());

					for (index, item) in layout.items.iter().enumerate() {
						let mut item_substitution = substitution.clone();
						item_substitution.intro(item.intro);

						let item_substitution = Matching::new(
							dataset,
							item_substitution,
							item.dataset.quads().with_default_graph(current_graph),
						)
						.into_required_unique()
						.for_fragment(|| DataFragment::SizedListItem {
							layout: layout_ref.clone().cast(),
							index,
						})?;

						let item_inputs = select_inputs(&item.value.input, &item_substitution);
						let item_graph =
							select_graph(current_graph, &item.value.graph, &item_substitution);

						let item = hydrate_with_ref(
							vocabulary,
							interpretation,
							context,
							dataset,
							item_graph.as_ref(),
							&item.value.layout,
							&item_inputs,
						)?;

						items.push(item)
					}

					Ok(TypedValue::List(items, layout_ref.casted()))
				}
			}
		}
		Layout::Always => Ok(TypedValue::Always(Value::unit())),
	}
}

fn select_inputs<R: Clone>(inputs: &[Pattern<R>], substitution: &Substitution<R>) -> Vec<R> {
	inputs
		.iter()
		.map(|p| p.apply(substitution).into_resource().unwrap())
		.collect()
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
