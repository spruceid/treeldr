use std::{
	collections::{HashMap, HashSet},
	hash::Hash,
};

use crate::{
	layout::{DataLayout, LayoutType, ListLayout, LiteralLayout},
	Layout, Layouts, Literal, Pattern, Ref, Value, ValueFormat,
};
use grdf::BTreeDataset;
use iref::IriBuf;
use rdf_types::{
	generator, Id, Interpretation, InterpretationMut, IriVocabulary, LanguageTagVocabulary,
	LiteralVocabulary, Quad, ReverseIriInterpretation, ReverseIriInterpretationMut,
	ReverseLiteralInterpretation, ReverseLiteralInterpretationMut, Term, VocabularyMut,
};

use super::RdfContextMut;

mod data;

pub type RdfLiteralType<V> =
	rdf_types::literal::Type<<V as IriVocabulary>::Iri, <V as LanguageTagVocabulary>::LanguageTag>;
pub type RdfLiteral<V> = rdf_types::Literal<RdfLiteralType<V>, <V as LiteralVocabulary>::Value>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

/// Deserialize the given `value` according to the provided `layout`, returning
/// the deserialized RDF dataset.
pub fn dehydrate(
	layouts: &Layouts,
	value: &Value,
	layout_ref: &Ref<LayoutType>,
	expected_input_count: Option<u32>,
) -> Result<(BTreeDataset<Term>, Vec<Term>), Error> {
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
		fn assign_iri(&mut self, id: Self::Resource, iri: Self::Iri) -> bool {
			self.map.entry(id).or_default().insert(Term::iri(iri))
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
		fn assign_literal(&mut self, id: Self::Resource, literal: Self::Literal) -> bool {
			self.map
				.entry(id)
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

	let layout = layouts.get(layout_ref).unwrap();
	let input_count = layout
		.input_count()
		.unwrap_or(expected_input_count.unwrap_or(1)) as usize;
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
	let mut generator = generator::Blank::new();

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

	fn map_resource(
		generator: &mut generator::Blank,
		map: &mut HashMap<InputResource, Term>,
		r: InputResource,
	) -> Term {
		match r {
			InputResource::Term(t) => t,
			r => map
				.entry(r)
				.or_insert_with(|| Term::blank(generator.next_blank_id()))
				.clone(),
		}
	}

	let dataset = dataset
		.into_iter()
		.map(|quad| {
			Quad(
				map_resource(&mut generator, &mut map, quad.0),
				map_resource(&mut generator, &mut map, quad.1),
				map_resource(&mut generator, &mut map, quad.2),
				quad.3.map(|g| map_resource(&mut generator, &mut map, g)),
			)
		})
		.collect();

	let values = (0..input_count)
		.map(|i| map.get(&InputResource::Input(i)).unwrap().clone())
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
) -> Result<(), Error>
where
	V: VocabularyMut<Type = RdfLiteralType<V>>,
	V::Iri: Clone,
	V::Value: From<String>,
	I: InterpretationMut<V>
		+ ReverseIriInterpretationMut<Iri = V::Iri>
		+ ReverseLiteralInterpretationMut<Literal = V::Literal>,
	I::Resource: Clone + Ord,
	Q: Clone + Ord + Into<I::Resource>,
	D: grdf::MutableDataset<
		Subject = I::Resource,
		Predicate = I::Resource,
		Object = I::Resource,
		GraphLabel = I::Resource,
	>,
{
	let layout = layouts.get(layout_ref).unwrap();

	if let Some(expected) = layout.input_count().filter(|&i| i != inputs.len() as u32) {
		return Err(Error::InvalidInputCount {
			expected,
			found: inputs.len() as u32,
		});
	}

	let env = Environment::Root(inputs);

	match layout {
		Layout::Never => Err(Error::IncompatibleLayout),
		Layout::Literal(layout) => match value {
			Value::Literal(value) => match layout {
				LiteralLayout::Data(layout) => match (layout, value) {
					(DataLayout::Unit(layout), Literal::Unit) => {
						let env = env.intro(rdf, layout.intro);
						env.instantiate_dataset(&layout.dataset, output)?;
						Ok(())
					}
					(DataLayout::Boolean(layout), Literal::Boolean(value)) => {
						let env = env.intro(rdf, layout.intro);
						env.instantiate_dataset(&layout.dataset, output)?;
						let resource = env.instantiate_pattern(&layout.resource)?;

						let literal =
							data::dehydrate_boolean(rdf, *value, &layout.datatype.clone().into())?;
						rdf.interpretation
							.assign_literal(resource, rdf.vocabulary.insert_owned_literal(literal));

						Ok(())
					}
					(DataLayout::Number(layout), Literal::Number(value)) => {
						let env = env.intro(rdf, layout.intro);
						env.instantiate_dataset(&layout.dataset, output)?;
						let resource = env.instantiate_pattern(&layout.resource)?;

						let literal =
							data::dehydrate_number(rdf, value, &layout.datatype.clone().into())?;
						rdf.interpretation
							.assign_literal(resource, rdf.vocabulary.insert_owned_literal(literal));

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
						rdf.interpretation
							.assign_literal(resource, rdf.vocabulary.insert_owned_literal(literal));

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
						rdf.interpretation
							.assign_literal(resource, rdf.vocabulary.insert_owned_literal(literal));

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
								rdf.interpretation.assign_iri(resource, i);
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
) -> Result<(), Error>
where
	V: VocabularyMut<Type = RdfLiteralType<V>>,
	V::Iri: Clone,
	V::Value: From<String>,
	I: InterpretationMut<V>
		+ ReverseIriInterpretationMut<Iri = V::Iri>
		+ ReverseLiteralInterpretationMut<Literal = V::Literal>,
	I::Resource: Clone + Ord,
	Q: Clone + Ord + Into<I::Resource>,
	D: grdf::MutableDataset<
		Subject = I::Resource,
		Predicate = I::Resource,
		Object = I::Resource,
		GraphLabel = I::Resource,
	>,
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
			Self::Root(inputs) => inputs
				.get(i as usize)
				.ok_or_else(|| i - inputs.len() as u32),
			Self::Child(parent, intros) => match parent.get(i) {
				Ok(r) => Ok(r),
				Err(j) => intros
					.get(j as usize)
					.ok_or_else(|| j - intros.len() as u32),
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
	pub fn instantiate_pattern<Q>(&self, pattern: &Pattern<Q>) -> Result<R, Error>
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

	pub fn instantiate_patterns<Q>(&self, patterns: &[Pattern<Q>]) -> Result<Vec<R>, Error>
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
	) -> Result<Quad<R, R, R, R>, Error>
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
	) -> Result<(), Error>
	where
		Q: Clone + Into<R>,
		D: grdf::MutableDataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
	{
		for quad in input.quads() {
			output.insert(self.instantiate_quad(quad)?);
		}

		Ok(())
	}
}
