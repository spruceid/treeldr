use std::{
	collections::{BTreeMap, HashMap, HashSet},
	hash::Hash,
};

use crate::{
	layout::{DataLayout, LayoutType, ListLayout, LiteralLayout, ProductLayoutType},
	Layout, Layouts, Literal, Pattern, Ref, Value, ValueFormat,
};
use iref::IriBuf;
use rdf_types::{
	dataset::{BTreeDataset, DatasetMut, TraversableDataset},
	generator,
	interpretation::{
		ReverseIriInterpretation, ReverseIriInterpretationMut, ReverseLiteralInterpretation,
		ReverseLiteralInterpretationMut,
	},
	vocabulary::IriVocabulary,
	BlankIdBuf, Generator, Id, Interpretation, InterpretationMut, Quad, Term, VocabularyMut,
};

use super::RdfContextMut;

mod data;

pub type RdfLiteral<V> = rdf_types::Literal<<V as IriVocabulary>::Iri>;

/// Dehydrate error.
#[derive(Debug, thiserror::Error)]
pub enum Error<R = Term> {
	#[error("incompatible layout")]
	IncompatibleLayout,

	#[error("invalid input count (expected {expected}, found {found})")]
	InvalidInputCount { expected: u32, found: u32 },

	#[error("undeclared variable #{0}")]
	UndeclaredVariable(u32),

	#[error("data ambiguity")]
	DataAmbiguity,

	#[error(transparent)]
	TermAmbiguity(Box<TermAmbiguity>),

	#[error("layout {0} not found")]
	LayoutNotFound(Ref<LayoutType, R>),

	#[error("missing required field `{field_name}`")]
	MissingRequiredField {
		layout: Ref<ProductLayoutType, R>,
		field_name: String,
		value: BTreeMap<String, Value>,
	},
}

impl<R> Error<R> {
	pub fn map_ids<S>(self, f: impl Fn(R) -> S) -> Error<S> {
		match self {
			Self::IncompatibleLayout => Error::IncompatibleLayout,
			Self::InvalidInputCount { expected, found } => {
				Error::InvalidInputCount { expected, found }
			}
			Self::UndeclaredVariable(x) => Error::UndeclaredVariable(x),
			Self::DataAmbiguity => Error::DataAmbiguity,
			Self::TermAmbiguity(a) => Error::TermAmbiguity(a),
			Self::LayoutNotFound(layout_ref) => Error::LayoutNotFound(layout_ref.map(f)),
			Self::MissingRequiredField {
				layout,
				field_name,
				value,
			} => Error::MissingRequiredField {
				layout: layout.map(f),
				field_name,
				value,
			},
		}
	}
}

#[derive(Debug, thiserror::Error)]
#[error("term ambiguity ({a} or {b})")]
pub struct TermAmbiguity {
	pub a: Term,
	pub b: Term,
}

impl TermAmbiguity {
	pub fn new(a: Term, b: Term) -> Box<Self> {
		Box::new(Self { a, b })
	}
}

/// Options to the [`dehydrate`] function.
///
/// Most of the time `Options::default()` should work as expected. You can
/// tweak the options to have more control on the expected number of inputs
/// to the layout and how RDF terms are associated to anonymous resources (
/// resources that don't have terms defined in the input tree value).
pub struct Options<G = generator::Blank> {
	/// The number of input resources passed to the deserialization function
	/// (and hence the size of the output `Vec<Term>`).
	///
	/// If `None`, the input count is decided using the required input count of
	/// the layout. However for the top and bottom layouts, that don't have a
	/// required input count, only one input resource is passed by default
	/// which may not fit all use cases. This option can be used to set the
	/// input count manually.
	input_count: Option<u32>,

	/// Defines what term is given to input resources when none are defined by
	/// the input tree value.
	///
	/// By default all input resources are associated to the blank node
	/// identifier `_:input{i}` where `{i}` is replaced by the input index.
	input_term_generator: fn(usize) -> Term,

	/// Resource id generator for non-input resources.
	///
	/// By default the [`generator::Blank`] generator is used, creating a new
	/// fresh blank node identifier of the form `_:{i}` for each resource,
	/// where `{i}` is replaced by a unique number from `0` to `n` where `n` is
	/// the number of anonymous non-input resources.
	generator: G,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			input_count: None,
			input_term_generator: |i| Term::blank(BlankIdBuf::new(format!("_:input{i}")).unwrap()),
			generator: generator::Blank::new(),
		}
	}
}

impl<G> Options<G> {
	/// Changes the number of inputs passed to the deserialization algorithm.
	///
	/// When defined, the layout's input count is used. However for the top
	/// and bottom layouts that don't have an input count, `1` is used by
	/// default, which may not fit all use cases. This option can be used to
	/// set the input count manually.
	pub fn with_input_count(self, count: u32) -> Self {
		Self {
			input_count: Some(count),
			..self
		}
	}

	/// Defines what term is given to input resources when none are defined by
	/// the input tree value.
	///
	/// By default all input resources are associated to the blank node
	/// identifier `_:input{i}` where `{i}` is replaced by the input index.
	pub fn with_input_term_generator(self, f: fn(usize) -> Term) -> Self {
		Self {
			input_term_generator: f,
			..self
		}
	}

	/// Changes the generator used to generate non-input anonymous resources
	/// terms.
	///
	/// By default the [`generator::Blank`] generator is used, creating a new
	/// fresh blank node identifier of the form `_:{i}` for each resource,
	/// where `{i}` is replaced by a unique number from `0` to `n` where `n` is
	/// the number of anonymous non-input resources.
	pub fn with_generator<H>(self, generator: H) -> Options<H> {
		Options {
			input_count: self.input_count,
			input_term_generator: self.input_term_generator,
			generator,
		}
	}
}

/// Deserialize the given tree `value` into an RDF dataset.
///
/// This is a simplified version of [`dehydrate_with`] using the basic unit `()`
/// RDF interpretation where resources are interpreted as their lexical
/// representation (a [`Term`]).
///
/// Deserialization is performed according to the layout identified by
/// `layout_ref` in the layout collection `layouts`. The function returns the
/// RDF dataset represented by the input tree value alongside with a list of
/// terms corresponding to the lexical representation of the input resources
/// passed to the deserialization algorithm.
///
/// ```
/// use serde_json::json;
/// use rdf_types::{Term, Id, dataset::PatternMatchingDataset};
/// use static_iref::iri;
///
/// // Create a layout builder.
/// let mut builder = treeldr_layouts::abs::Builder::new();
///
/// // Create a layout from its abstract (JSON) syntax.
/// // This layout has a single (implicit) input (`_:self`).
/// let layout: treeldr_layouts::abs::syntax::Layout = serde_json::from_value(
///   json!({
///     "type": "record",
///     "fields": {
///       "id": {
///         "intro": [],
///         "value": {
///           "layout": { "type": "id" },
///           "input": "_:self"
///         },
///       },
///       "name": {
///         "value": { "type": "string" },
///         "property": "https://schema.org/name"
///       }
///     }
///   })
/// ).unwrap();
///
/// // Build the layout.
/// let layout_ref = layout.build(&mut builder).unwrap();
///
/// // Get the compiled layouts collection.
/// let layouts = builder.build();
///
/// let value: treeldr_layouts::Value = serde_json::from_value(
///   json!({
///     "id": "https://example.org/JohnSmith",
///     "name": "John Smith"
///   })
/// ).unwrap();
///
/// // Dehydrate!
/// let (dataset, subjects) = treeldr_layouts::distill::dehydrate(
///     &layouts,
///     &value,
///     &layout_ref,
///     treeldr_layouts::distill::de::Options::default()
/// ).unwrap();
///
/// // Index the dataset so we can run queries on it.
/// let dataset = dataset.into_indexed();
///
/// // The number of subjects is equal to the number of layout inputs.
/// assert_eq!(subjects.len(), 1);
///
/// // The only subject here is <https://example.org/JohnSmith>.
/// assert_eq!(subjects[0].as_iri().unwrap(), "https://example.org/JohnSmith");
///
/// // Fetch the name of our subject.
/// let name_pred = Term::Id(Id::Iri(iri!("https://schema.org/name").to_owned()));
/// let mut names = dataset.quad_objects(None, &subjects[0], &name_pred);
///
/// // Should match what is defined in the serialized value.
/// assert_eq!(names.next().unwrap().as_literal().unwrap().value, "John Smith")
/// ```
///
/// In the more generic [`dehydrate_with`], the input terms are given as input
/// to the function. However in this simplified version (where resources are
/// identified by their lexical representation) it is not possible to provide
/// the input resource in advance, since we don't know their lexical
/// representation. In the example above, that would mean providing the
/// `dehydrate` function with the `https://example.org/JohnSmith` IRI in advance
/// since it is the subject of the JSON document, but we don't know that yet.
/// Instead, this function will create a intermediate
/// interpretation of resources, allowing the term representation of the input
/// resources to be collected during deserialization. The collected terms
/// are then returned along with the RDF dataset.
pub fn dehydrate<G: Generator>(
	layouts: &Layouts,
	value: &Value,
	layout_ref: &Ref<LayoutType>,
	mut options: Options<G>,
) -> Result<(BTreeDataset, Vec<Term>), Error> {
	#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
	enum InputResource {
		Input(usize),
		Anonymous(usize),
		Term(Term),
	}

	impl InputResource {
		pub fn as_term(&self) -> Option<&Term> {
			match self {
				Self::Term(t) => Some(t),
				_ => None,
			}
		}
	}

	impl From<Term> for InputResource {
		fn from(value: Term) -> Self {
			Self::Term(value)
		}
	}

	struct InputInterpretation {
		map: HashMap<InputResource, HashSet<Term>>,
		anonymous_count: usize,
	}

	impl Interpretation for InputInterpretation {
		type Resource = InputResource;
	}

	impl InterpretationMut<()> for InputInterpretation {
		fn new_resource(&mut self, _vocabulary: &mut ()) -> Self::Resource {
			let i = self.anonymous_count;
			self.anonymous_count += 1;
			InputResource::Anonymous(i)
		}
	}

	impl ReverseIriInterpretation for InputInterpretation {
		type Iri = IriBuf;
		type Iris<'a> = IrisOf<'a>;

		fn iris_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Iris<'a> {
			IrisOf {
				term: id.as_term(),
				additional_terms: self.map.get(id).map(|t| t.iter()),
			}
		}
	}

	impl ReverseIriInterpretationMut for InputInterpretation {
		fn assign_iri(&mut self, id: &Self::Resource, iri: Self::Iri) -> bool {
			self.map
				.entry(id.clone())
				.or_default()
				.insert(Term::iri(iri))
		}
	}

	impl ReverseLiteralInterpretation for InputInterpretation {
		type Literal = RdfLiteral<()>;
		type Literals<'a> = LiteralsOf<'a>;

		fn literals_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Literals<'a> {
			LiteralsOf {
				term: id.as_term(),
				additional_terms: self.map.get(id).map(|t| t.iter()),
			}
		}
	}

	impl ReverseLiteralInterpretationMut for InputInterpretation {
		fn assign_literal(&mut self, id: &Self::Resource, literal: Self::Literal) -> bool {
			self.map
				.entry(id.clone())
				.or_default()
				.insert(Term::Literal(literal))
		}
	}

	#[derive(Debug, Clone)]
	pub struct IrisOf<'a> {
		term: Option<&'a Term>,
		additional_terms: Option<std::collections::hash_set::Iter<'a, Term>>,
	}

	impl<'a> Iterator for IrisOf<'a> {
		type Item = &'a IriBuf;

		fn next(&mut self) -> Option<Self::Item> {
			match self.term.take() {
				Some(Term::Id(Id::Iri(iri))) => Some(iri),
				_ => match self.additional_terms.as_mut() {
					Some(terms) => {
						for term in terms {
							if let Term::Id(Id::Iri(iri)) = term {
								return Some(iri);
							}
						}

						None
					}
					None => None,
				},
			}
		}
	}

	#[derive(Debug, Clone)]
	pub struct LiteralsOf<'a> {
		term: Option<&'a Term>,
		additional_terms: Option<std::collections::hash_set::Iter<'a, Term>>,
	}

	impl<'a> Iterator for LiteralsOf<'a> {
		type Item = &'a RdfLiteral<()>;

		fn next(&mut self) -> Option<Self::Item> {
			match self.term.take() {
				Some(Term::Literal(l)) => Some(l),
				_ => match self.additional_terms.as_mut() {
					Some(terms) => {
						for term in terms {
							if let Term::Literal(l) = term {
								return Some(l);
							}
						}

						None
					}
					None => None,
				},
			}
		}
	}

	let layout = layouts
		.get(layout_ref)
		.ok_or_else(|| Error::LayoutNotFound(layout_ref.clone()))?;
	let input_count = layout
		.input_count()
		.unwrap_or(options.input_count.unwrap_or(1)) as usize;
	let mut inputs = Vec::with_capacity(input_count);
	for i in 0..input_count {
		inputs.push(InputResource::Input(i))
	}

	let mut dataset = BTreeDataset::new();

	let mut interpretation = InputInterpretation {
		map: HashMap::new(),
		anonymous_count: 0,
	};

	let mut rdf = RdfContextMut {
		vocabulary: &mut (),
		interpretation: &mut interpretation,
	};

	dehydrate_with(
		&mut rdf,
		layouts,
		value,
		None,
		layout_ref,
		&inputs,
		&mut dataset,
	)?;

	let mut map = HashMap::new();

	for (r, terms) in interpretation.map {
		match r {
			InputResource::Term(t) => {
				for u in terms {
					if t != u {
						return Err(Error::TermAmbiguity(TermAmbiguity::new(t, u)));
					}
				}
			}
			r => {
				let mut value = None;

				for term in terms {
					if let Some(t) = value.replace(term) {
						return Err(Error::TermAmbiguity(TermAmbiguity::new(t, value.unwrap())));
					}
				}

				if let Some(value) = value {
					map.insert(r, value);
				}
			}
		}
	}

	fn map_resource<G: Generator>(
		map: &mut HashMap<InputResource, Term>,
		r: InputResource,
		options: &mut Options<G>,
	) -> Term {
		match r {
			InputResource::Term(t) => t,
			InputResource::Input(i) => map
				.entry(InputResource::Input(i))
				.or_insert_with(|| (options.input_term_generator)(i))
				.clone(),
			InputResource::Anonymous(i) => map
				.entry(InputResource::Anonymous(i))
				.or_insert_with(|| Term::Id(options.generator.next(&mut ())))
				.clone(),
		}
	}

	let dataset = dataset
		.into_iter()
		.map(|quad| {
			Quad(
				map_resource(&mut map, quad.0, &mut options),
				map_resource(&mut map, quad.1, &mut options),
				map_resource(&mut map, quad.2, &mut options),
				quad.3.map(|g| map_resource(&mut map, g, &mut options)),
			)
		})
		.collect();

	let values = (0..input_count)
		.map(|i| map_resource(&mut map, InputResource::Input(i), &mut options))
		.collect();

	Ok((dataset, values))
}

/// Deserialize the given `value` according to the provided `layout`, returning
/// the deserialized RDF dataset.
pub fn dehydrate_with<V, I, Q, D>(
	rdf: &mut RdfContextMut<V, I>,
	layouts: &Layouts<Q>,
	value: &Value,
	current_graph: Option<&I::Resource>,
	layout_ref: &Ref<LayoutType, Q>,
	inputs: &[I::Resource],
	output: &mut D,
) -> Result<(), Error<Q>>
where
	V: VocabularyMut,
	V::Iri: Clone,
	I: InterpretationMut<V>
		+ ReverseIriInterpretationMut<Iri = V::Iri>
		+ ReverseLiteralInterpretationMut<Literal = V::Literal>,
	I::Resource: Clone + Ord,
	Q: Clone + Ord + Into<I::Resource>,
	D: TraversableDataset<Resource = I::Resource> + DatasetMut,
{
	let layout = layouts
		.get(layout_ref)
		.ok_or_else(|| Error::LayoutNotFound(layout_ref.clone()))?;

	if let Some(expected) = layout.input_count().filter(|&i| i != inputs.len() as u32) {
		return Err(Error::InvalidInputCount {
			expected,
			found: inputs.len() as u32,
		});
	}

	let env = Environment::Root(inputs);

	match layout {
		Layout::Never => Err(Error::IncompatibleLayout),
		Layout::Literal(LiteralLayout::Data(DataLayout::Unit(layout)))
			if *value == layout.const_ =>
		{
			let env = env.intro(rdf, layout.intro);
			env.instantiate_dataset(&layout.dataset, output)?;
			Ok(())
		}
		Layout::Literal(layout) => match value {
			Value::Literal(value) => match layout {
				LiteralLayout::Data(layout) => match (layout, value) {
					(DataLayout::Boolean(layout), Literal::Boolean(value)) => {
						let env = env.intro(rdf, layout.intro);
						env.instantiate_dataset(&layout.dataset, output)?;
						let resource = env.instantiate_pattern(&layout.resource)?;

						let literal =
							data::dehydrate_boolean(rdf, *value, &layout.datatype.clone().into())?;
						rdf.interpretation.assign_literal(
							&resource,
							rdf.vocabulary.insert_owned_literal(literal),
						);

						Ok(())
					}
					(DataLayout::Number(layout), Literal::Number(value)) => {
						let env = env.intro(rdf, layout.intro);
						env.instantiate_dataset(&layout.dataset, output)?;
						let resource = env.instantiate_pattern(&layout.resource)?;

						let literal =
							data::dehydrate_number(rdf, value, &layout.datatype.clone().into())?;
						rdf.interpretation.assign_literal(
							&resource,
							rdf.vocabulary.insert_owned_literal(literal),
						);

						Ok(())
					}
					(DataLayout::ByteString(layout), Literal::ByteString(value)) => {
						let env = env.intro(rdf, layout.intro);
						env.instantiate_dataset(&layout.dataset, output)?;
						let resource = env.instantiate_pattern(&layout.resource)?;

						let literal = data::dehydrate_byte_string(
							rdf,
							value,
							&layout.datatype.clone().into(),
						)?;
						rdf.interpretation.assign_literal(
							&resource,
							rdf.vocabulary.insert_owned_literal(literal),
						);

						Ok(())
					}
					(DataLayout::TextString(layout), Literal::TextString(value)) => {
						let env = env.intro(rdf, layout.intro);
						env.instantiate_dataset(&layout.dataset, output)?;
						let resource = env.instantiate_pattern(&layout.resource)?;

						let literal = data::dehydrate_text_string(
							rdf,
							value,
							&layout.datatype.clone().into(),
						)?;
						rdf.interpretation.assign_literal(
							&resource,
							rdf.vocabulary.insert_owned_literal(literal),
						);

						Ok(())
					}
					_ => Err(Error::IncompatibleLayout),
				},
				LiteralLayout::Id(layout) => match value {
					Literal::TextString(value) => {
						let env = env.intro(rdf, layout.intro);
						env.instantiate_dataset(&layout.dataset, output)?;
						let resource = env.instantiate_pattern(&layout.resource)?;

						match IriBuf::new(value.to_owned()) {
							Ok(iri) => {
								let i = rdf.vocabulary.insert_owned(iri);
								rdf.interpretation.assign_iri(&resource, i);
								Ok(())
							}
							Err(_) => Err(Error::IncompatibleLayout), // not an IRI
						}
					}
					_ => Err(Error::IncompatibleLayout),
				},
			},
			_ => Err(Error::IncompatibleLayout),
		},
		Layout::Sum(layout) => {
			let env = env.intro(rdf, layout.intro);
			env.instantiate_dataset(&layout.dataset, output)?;

			let mut selection = None;
			for variant in &layout.variants {
				let mut variant_dataset = BTreeDataset::new();

				let env = env.intro(rdf, variant.intro);
				env.instantiate_dataset(&variant.dataset, &mut variant_dataset)?;

				if dehydrate_sub_value(
					rdf,
					layouts,
					value,
					current_graph,
					&variant.value,
					&env,
					output,
				)
				.is_ok() && selection.replace(variant_dataset).is_some()
				{
					return Err(Error::DataAmbiguity);
				}
			}

			match selection {
				Some(variant_dataset) => {
					for quad in variant_dataset {
						output.insert(quad);
					}

					Ok(())
				}
				None => Err(Error::IncompatibleLayout),
			}
		}
		Layout::Product(layout) => match value {
			Value::Record(value) => {
				let env = env.intro(rdf, layout.intro);
				env.instantiate_dataset(&layout.dataset, output)?;

				for (key, value) in value {
					match layout.fields.get(key) {
						Some(field) => {
							let env = env.intro(rdf, field.intro);
							env.instantiate_dataset(&field.dataset, output)?;
							dehydrate_sub_value(
								rdf,
								layouts,
								value,
								current_graph,
								&field.value,
								&env,
								output,
							)?;
						}
						None => return Err(Error::IncompatibleLayout),
					}
				}

				for (name, field) in &layout.fields {
					if field.required && !value.contains_key(name) {
						return Err(Error::MissingRequiredField {
							layout: layout_ref.clone().cast(),
							field_name: name.clone(),
							value: value.clone(),
						});
					}
				}

				Ok(())
			}
			_ => Err(Error::IncompatibleLayout),
		},
		Layout::List(layout) => match value {
			Value::List(value) => match layout {
				ListLayout::Unordered(layout) => {
					let env = env.intro(rdf, layout.intro);
					env.instantiate_dataset(&layout.dataset, output)?;

					for item in value {
						let env = env.intro(rdf, layout.item.intro);
						env.instantiate_dataset(&layout.item.dataset, output)?;
						dehydrate_sub_value(
							rdf,
							layouts,
							item,
							current_graph,
							&layout.item.value,
							&env,
							output,
						)?;
					}

					Ok(())
				}
				ListLayout::Ordered(layout) => {
					let env = env.intro(rdf, layout.intro);
					env.instantiate_dataset(&layout.dataset, output)?;

					let mut head = env.instantiate_pattern(&layout.head)?;

					for i in 0..value.len() {
						let rest = if i == value.len() - 1 {
							env.instantiate_pattern(&layout.tail)?
						} else {
							rdf.interpretation.new_resource(rdf.vocabulary)
						};

						let env = env.bind([head, rest.clone()]);
						let env = env.intro(rdf, layout.node.intro);
						env.instantiate_dataset(&layout.node.dataset, output)?;

						let item = &value[i];
						dehydrate_sub_value(
							rdf,
							layouts,
							item,
							current_graph,
							&layout.node.value,
							&env,
							output,
						)?;

						head = rest;
					}

					Ok(())
				}
				ListLayout::Sized(layout) => {
					let env = env.intro(rdf, layout.intro);
					env.instantiate_dataset(&layout.dataset, output)?;

					let mut items = value.iter();
					let mut item_layouts = layout.items.iter();

					loop {
						match (items.next(), item_layouts.next()) {
							(Some(item), Some(item_layout)) => {
								let env = env.intro(rdf, item_layout.intro);
								env.instantiate_dataset(&item_layout.dataset, output)?;
								dehydrate_sub_value(
									rdf,
									layouts,
									item,
									current_graph,
									&item_layout.value,
									&env,
									output,
								)?;
							}
							(None, None) => break,
							_ => return Err(Error::IncompatibleLayout),
						}
					}

					Ok(())
				}
			},
			_ => Err(Error::IncompatibleLayout),
		},
		Layout::Always => Ok(()),
	}
}

fn dehydrate_sub_value<V, I, Q, D>(
	rdf: &mut RdfContextMut<V, I>,
	layouts: &Layouts<Q>,
	value: &Value,
	current_graph: Option<&I::Resource>,
	format: &ValueFormat<Q>,
	env: &Environment<I::Resource>,
	output: &mut D,
) -> Result<(), Error<Q>>
where
	V: VocabularyMut,
	V::Iri: Clone,
	I: InterpretationMut<V>
		+ ReverseIriInterpretationMut<Iri = V::Iri>
		+ ReverseLiteralInterpretationMut<Literal = V::Literal>,
	I::Resource: Clone + Ord,
	Q: Clone + Ord + Into<I::Resource>,
	D: TraversableDataset<Resource = I::Resource> + DatasetMut,
{
	let inputs = env.instantiate_patterns(&format.input)?;
	let graph = match &format.graph {
		Some(None) => None,
		Some(Some(g)) => Some(env.instantiate_pattern(g)?),
		None => current_graph.cloned(),
	};

	dehydrate_with(
		rdf,
		layouts,
		value,
		graph.as_ref(),
		&format.layout,
		&inputs,
		output,
	)
}

pub enum Environment<'a, R> {
	Root(&'a [R]),
	Child(&'a Environment<'a, R>, Vec<R>),
}

impl<'a, R> Environment<'a, R> {
	pub fn get(&self, i: u32) -> Result<&R, u32> {
		match self {
			Self::Root(inputs) => match inputs.get(i as usize) {
				Some(r) => Ok(r),
				None => Err(i - inputs.len() as u32),
			},
			Self::Child(parent, intros) => match parent.get(i) {
				Ok(r) => Ok(r),
				Err(j) => match intros.get(j as usize) {
					Some(r) => Ok(r),
					None => Err(j - intros.len() as u32),
				},
			},
		}
	}

	#[must_use]
	pub fn bind<const N: usize>(&self, resources: [R; N]) -> Environment<R> {
		Environment::Child(self, resources.into_iter().collect())
	}

	#[must_use]
	pub fn intro<V, I>(&self, rdf: &mut RdfContextMut<V, I>, count: u32) -> Environment<R>
	where
		I: InterpretationMut<V, Resource = R>,
	{
		let mut intros = Vec::with_capacity(count as usize);
		for _ in 0..count {
			intros.push(rdf.interpretation.new_resource(rdf.vocabulary))
		}

		Environment::Child(self, intros)
	}
}

impl<'a, R: Clone> Environment<'a, R> {
	pub fn instantiate_pattern<Q>(&self, pattern: &Pattern<Q>) -> Result<R, Error<Q>>
	where
		Q: Clone + Into<R>,
	{
		match pattern {
			Pattern::Var(x) => self
				.get(*x)
				.cloned()
				.map_err(|_| Error::UndeclaredVariable(*x)),
			Pattern::Resource(r) => Ok(r.clone().into()),
		}
	}

	pub fn instantiate_patterns<Q>(&self, patterns: &[Pattern<Q>]) -> Result<Vec<R>, Error<Q>>
	where
		Q: Clone + Into<R>,
	{
		let mut result = Vec::with_capacity(patterns.len());

		for p in patterns {
			result.push(self.instantiate_pattern(p)?)
		}

		Ok(result)
	}

	pub fn instantiate_quad<Q>(
		&self,
		quad: Quad<&Pattern<Q>, &Pattern<Q>, &Pattern<Q>, &Pattern<Q>>,
	) -> Result<Quad<R, R, R, R>, Error<Q>>
	where
		Q: Clone + Into<R>,
	{
		Ok(Quad(
			self.instantiate_pattern(quad.0)?,
			self.instantiate_pattern(quad.1)?,
			self.instantiate_pattern(quad.2)?,
			quad.3.map(|g| self.instantiate_pattern(g)).transpose()?,
		))
	}

	pub fn instantiate_dataset<Q, D>(
		&self,
		input: &BTreeDataset<Pattern<Q>>,
		output: &mut D,
	) -> Result<(), Error<Q>>
	where
		Q: Clone + Into<R>,
		D: TraversableDataset<Resource = R> + DatasetMut,
	{
		for quad in input.quads() {
			output.insert(self.instantiate_quad(quad)?);
		}

		Ok(())
	}
}
