use std::{
	collections::BTreeMap,
	marker::PhantomData,
};

use iref::Iri;
use rdf_types::{Interpretation, Quad, ReverseIriInterpretation, ReverseLiteralInterpretation, Vocabulary, IriVocabulary, LanguageTagVocabulary};
use treeldr::{
	layout::{LayoutType, ListLayout, LiteralLayout, DataLayout},
	pattern::Substitution,
	Context, Layout, Pattern, Ref,
};

use crate::{TypedValue, TypedLiteral};

pub enum Error {
	IncompatibleLayout,
	AbstractLayout,
	InvalidInputCount { expected: u32, found: u32 },
}

pub type RdfLiteralType<V> = rdf_types::literal::Type<<V as IriVocabulary>::Iri, <V as LanguageTagVocabulary>::LanguageTag>;

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
	V: Vocabulary<Type = RdfLiteralType<V>>,
	V::Iri: PartialEq,
	V::Value: AsRef<str>,
	I: ReverseIriInterpretation<Iri = V::Iri> + ReverseLiteralInterpretation<Literal = V::Literal>,
	I::Resource: Clone + PartialEq,
	D: grdf::Dataset<
		Subject = I::Resource,
		Predicate = I::Resource,
		Object = I::Resource,
		GraphLabel = I::Resource,
	>,
{
	let layout = context.get(layout_ref).unwrap();

	if let Some(expected) = layout.input_count().filter(|&i| i != inputs.len() as u32) {
		return Err(Error::InvalidInputCount {
			expected,
			found: inputs.len() as u32,
		})
	}

	match context.get(layout_ref).unwrap() {
		Layout::Never => Err(Error::IncompatibleLayout),
		Layout::Literal(layout) => {
			let id = &inputs[0];

			match layout {
				LiteralLayout::Data(layout) => {
					hydrate_data(layout)
				}
				LiteralLayout::Id(_) => {
					for i in interpretation.iris_of(id) {
						// ...
					}

					todo!() // Error no IRI matching the layout
				}
			}
		}
		Layout::Sum(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);
			let substitution = Matching::new(
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph)
			).into_required_unique()?;

			let mut failures = Vec::new();
			let mut selected = None;

			for (i, variant) in layout.variants.iter().enumerate() {
				let mut variant_substitution = substitution.clone();
				variant_substitution.intro(variant.intro);

				let variant_substitution = Matching::new(
					substitution.clone(),
					variant.dataset.quads().with_default_graph(current_graph),
				).into_unique()?;

				match variant_substitution {
					Some(variant_substitution) => {
						let variant_inputs = select_inputs(
							&variant.format.inputs,
							&variant_substitution
						);
		
						let variant_graph = select_graph(
							current_graph,
							&variant.format.graph,
							&variant_substitution,
						);
		
						let value = hydrate(
							vocabulary,
							interpretation,
							context,
							dataset,
							variant_graph.as_ref(),
							&variant.format.layout,
							&variant_inputs,
						);
	
						match value {
							Ok(value) => {
								match selected.take() {
									Some((j, other_value)) => {
										todo!() // Error: variant ambiguity
									}
									None => {
										selected = Some((i, value))
									}
								}
							}
							Err(e) => {
								failures.push(Some(e))
							}
						}
					}
					None => {
						failures.push(None)
					}
				}
			}

			match selected {
				Some((i, value)) => {
					Ok(TypedValue::Variant(Box::new(value), layout_ref.casted(), i as u32))
				}
				None => {
					todo!() // Error: no variant found
				}
			}
		}
		Layout::Product(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);
			let substitution = Matching::new(
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph)
			).into_required_unique()?;

			let mut record = BTreeMap::new();

			for field in &layout.fields {
				let mut field_substitution = substitution.clone();
				field_substitution.intro(field.intro);

				let field_substitution = Matching::new(
					substitution.clone(),
					field.dataset.quads().with_default_graph(current_graph),
				).into_unique()?;

				match field_substitution {
					Some(field_substitution) => {
						let field_inputs = select_inputs(
							&field.format.inputs,
							&field_substitution
						);
		
						let item_graph = select_graph(
							current_graph,
							&field.format.graph,
							&field_substitution,
						);
		
						let value = hydrate(
							vocabulary,
							interpretation,
							context,
							dataset,
							item_graph.as_ref(),
							&field.format.layout,
							&field_inputs,
						)?;
		
						record.insert(field.name.clone(), value);
					}
					None => {
						// TODO check required fields
					}
				}
			}

			Ok(TypedValue::Record(record, layout_ref.casted()))
		}
		Layout::List(layout) => {
			match layout {
				ListLayout::Unordered(layout) => {
					let mut substitution = Substitution::from_inputs(inputs);
					substitution.intro(layout.intro);

					let mut item_substitution = Matching::new(
						substitution,
						layout.dataset.quads().with_default_graph(current_graph),
					).into_required_unique()?;

					item_substitution.intro(layout.item.intro);
					let matching = Matching::new(
						item_substitution,
						layout
							.item
							.dataset
							.quads()
							.with_default_graph(current_graph),
					);

					let mut items = Vec::new();

					for item_substitution in matching {
						let item_inputs = select_inputs(
							&layout.item.format.inputs,
							&item_substitution
						);

						let item_graph = select_graph(
							current_graph,
							&layout.item.format.graph,
							&item_substitution,
						);

						let item = hydrate(
							vocabulary,
							interpretation,
							context,
							dataset,
							item_graph.as_ref(),
							&layout.item.format.layout,
							&item_inputs,
						)?;

						items.push(item);
					}

					Ok(TypedValue::List(items, layout_ref.casted()))
				}
				ListLayout::Ordered(layout) => {
					let mut substitution = Substitution::from_inputs(inputs);
					substitution.intro(layout.intro);

					let substitution = Matching::new(
						substitution,
						layout.dataset.quads().with_default_graph(current_graph),
					).into_required_unique()?;

					let mut head = layout.head.apply(&substitution).into_resource().unwrap();
					let tail = layout.tail.apply(&substitution).into_resource().unwrap();

					let mut items = Vec::new();

					while head != tail {
						let mut item_substitution = substitution.clone();
						item_substitution.push(Some(head)); // the head
						let rest = item_substitution.intro(1 + layout.node.intro); // the rest, and other intro variables.

						let item_substitution = Matching::new(
							item_substitution,
							layout
								.node
								.dataset
								.quads()
								.with_default_graph(current_graph),
						).into_required_unique()?;

						let item_inputs = select_inputs(
							&layout.node.format.inputs,
							&item_substitution
						);

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

					Ok(TypedValue::List(items, layout_ref.casted()))
				}
				ListLayout::Sized(layout) => {
					let mut substitution = Substitution::from_inputs(inputs);
					substitution.intro(layout.intro);

					let substitution = Matching::new(
						substitution,
						layout.dataset.quads().with_default_graph(current_graph),
					).into_required_unique()?;

					let mut items = Vec::with_capacity(layout.items.len());

					for item in &layout.items {
						let mut item_substitution = substitution.clone();
						item_substitution.intro(item.intro);

						let item_substitution = Matching::new(
							item_substitution,
							item.dataset.quads().with_default_graph(current_graph)
						).into_required_unique()?;

						let item_inputs = select_inputs(&item.format.inputs, &item_substitution);
						let item_graph =
							select_graph(current_graph, &item.format.graph, &item_substitution);

						let item = hydrate(
							vocabulary,
							interpretation,
							context,
							dataset,
							item_graph.as_ref(),
							&item.format.layout,
							&item_inputs,
						)?;

						items.push(item)
					}

					Ok(TypedValue::List(items, layout_ref.casted()))
				}
			}
		}
		Layout::Always => Err(Error::AbstractLayout),
	}
}

fn hydrate_data<V, I: Interpretation>(
	vocabulary: &V,
	interpretation: &I,
	context: &Context<I::Resource>,
	dataset: &D,
	current_graph: Option<&I::Resource>,
	layout: &DataLayout<R>,
	inputs: &[I::Resource],
) -> Result<TypedLiteral<R>, Error>

{
	let value = match layout {
		DataLayout::Unit(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);
			
			Matching::new(
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph)
			).into_required_unique()?;

			TypedLiteral::Unit(layout_ref.casted())
		}
		DataLayout::Boolean(layout) => {
			let mut substitution = Substitution::from_inputs(inputs);
			substitution.intro(layout.intro);

			let substitution = Matching::new(
				substitution.clone(),
				layout.dataset.quads().with_default_graph(current_graph)
			).into_required_unique()?;

			let resource = layout.literal.resource.apply(&substitution).into_resource().unwrap();

			let mut value = None;

			for l in interpretation.literals_of(&resource) {
				let literal = vocabulary.literal(l).unwrap();
				let i = match literal.type_() {
					rdf_types::literal::Type::Any(i) => {
						i
					}
					rdf_types::literal::Type::LangString(_) => {
						todo!() // Lang string
					}
				};

				if interpretation.iris_of(&layout.literal.type_).any(|j| i == j) {
					let v = hydrate_boolean_value(
						literal.value().as_ref(),
						vocabulary.iri(i).unwrap(),
					)?;

					if value.replace(v).is_some() {
						todo!() // Ambiguity
					}
				}
			}

			match value {
				Some(value) => TypedLiteral::Boolean(value, layout_ref.casted()),
				None => {
					todo!() // No matching literal representation found
				}
			}
		}
		DataLayout::Number(_) => {
			todo!()
		}
		DataLayout::ByteString(_) => {
			todo!()
		}
		DataLayout::TextString(_) => {
			todo!()
		}
	};

	Ok(TypedValue::Literal(value))
}

fn select_inputs<R: Clone>(
	inputs: &[Pattern<R>],
	substitution: &Substitution<R>
) -> Vec<R> {
	inputs
	.iter()
	.map(|p| {
		p.apply(substitution)
			.into_resource()
			.unwrap()
	})
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

pub trait QuadsWithGraphExt<'a, R>: Sized {
	fn with_default_graph(self, graph: Option<&'a R>) -> QuadsWithGraph<'a, R, Self>;
}

impl<'a, R: 'a, I> QuadsWithGraphExt<'a, R> for I
where
	I: Iterator<Item = Quad<&'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>, &'a Pattern<R>>>,
{
	fn with_default_graph(self, graph: Option<&'a R>) -> QuadsWithGraph<'a, R, Self> {
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

	pub fn into_unique(self) -> Result<Option<Substitution<R>>, Error> {
		todo!()
	}

	pub fn into_required_unique(self) -> Result<Substitution<R>, Error> {
		todo!()
	}
}

impl<R, D> Iterator for Matching<R, D> {
	type Item = Substitution<R>;

	fn next(&mut self) -> Option<Self::Item> {
		todo!()
	}
}

fn hydrate_boolean_value(
	value: &str,
	type_: &Iri
) -> Result<bool, Error> {
	todo!()
}