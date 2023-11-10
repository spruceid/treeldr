use crate::{
	layout::{DataLayout, LayoutType, ListLayout, LiteralLayout},
	Layout, Layouts, Literal, Pattern, Ref, Value, ValueFormat,
};
use grdf::BTreeDataset;
use rdf_types::{InterpretationMut, Quad};

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
}

// /// Deserialize the given `value` according to the provided `layout`, returning
// /// the deserialized RDF dataset.
// pub fn dehydrate<V, I: Interpretation>(
// 	_vocabulary: &mut V,
// 	_interpretation: &mut I,
// 	layouts: &Layouts<I::Resource>,
// 	value: &Value,
// 	layout_ref: &Ref<LayoutType, I::Resource>,
// 	inputs: &[I::Resource]
// ) -> Result<BTreeDataset<I::Resource>, Error>
// where
// 	I::Resource: Ord,
// {
// 	dehydrate_in(vocabulary, interpretation, &mut dataset, layouts, value, layout_ref, inputs)?;
// 	Ok()
// }

/// Deserialize the given `value` according to the provided `layout`, returning
/// the deserialized RDF dataset.
pub fn dehydrate_with<V, I, D>(
	vocabulary: &mut V,
	interpretation: &mut I,
	layouts: &Layouts<I::Resource>,
	value: &Value,
	current_graph: Option<&I::Resource>,
	layout_ref: &Ref<LayoutType, I::Resource>,
	inputs: &[I::Resource],
	output: &mut D,
) -> Result<(), Error>
where
	I: InterpretationMut<V>,
	I::Resource: Clone + Ord,
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
						todo!()
					}
					(DataLayout::Boolean(layout), Literal::Boolean(value)) => {
						todo!()
					}
					(DataLayout::Number(layout), Literal::Number(value)) => {
						todo!()
					}
					(DataLayout::ByteString(layout), Literal::ByteString(value)) => {
						todo!()
					}
					(DataLayout::TextString(layout), Literal::TextString(value)) => {
						todo!()
					}
					_ => Err(Error::IncompatibleLayout),
				},
				LiteralLayout::Id(_) => match value {
					Literal::TextString(value) => {
						todo!()
					}
					_ => Err(Error::IncompatibleLayout),
				},
			},
			_ => Err(Error::IncompatibleLayout),
		},
		Layout::Sum(layout) => {
			let env = env.intro(vocabulary, interpretation, layout.intro);
			env.instantiate_dataset(&layout.dataset, output)?;

			let mut selection = None;
			for variant in &layout.variants {
				let mut variant_dataset = BTreeDataset::new();

				let env = env.intro(vocabulary, interpretation, variant.intro);
				env.instantiate_dataset(&variant.dataset, &mut variant_dataset)?;

				if dehydrate_sub_value(
					vocabulary,
					interpretation,
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
				let env = env.intro(vocabulary, interpretation, layout.intro);
				env.instantiate_dataset(&layout.dataset, output)?;

				for (key, value) in value {
					match layout.fields.get(key) {
						Some(field) => {
							let env = env.intro(vocabulary, interpretation, field.intro);
							env.instantiate_dataset(&field.dataset, output)?;
							dehydrate_sub_value(
								vocabulary,
								interpretation,
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
					let env = env.intro(vocabulary, interpretation, layout.intro);
					env.instantiate_dataset(&layout.dataset, output)?;

					for item in value {
						let env = env.intro(vocabulary, interpretation, layout.item.intro);
						dehydrate_sub_value(
							vocabulary,
							interpretation,
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
					let env = env.intro(vocabulary, interpretation, layout.intro);
					env.instantiate_dataset(&layout.dataset, output)?;

					let mut head = env.instantiate_pattern(&layout.head)?;

					for i in 0..value.len() {
						let rest = if i == value.len() - 1 {
							env.instantiate_pattern(&layout.tail)?
						} else {
							interpretation.new_resource(vocabulary)
						};

						let env = env.bind([head, rest.clone()]);
						let env = env.intro(vocabulary, interpretation, layout.node.intro);

						let item = &value[i];
						dehydrate_sub_value(
							vocabulary,
							interpretation,
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
					let env = env.intro(vocabulary, interpretation, layout.intro);
					env.instantiate_dataset(&layout.dataset, output)?;

					let mut items = value.iter();
					let mut item_layouts = layout.items.iter();

					loop {
						match (items.next(), item_layouts.next()) {
							(Some(item), Some(item_layout)) => {
								let env = env.intro(vocabulary, interpretation, item_layout.intro);
								dehydrate_sub_value(
									vocabulary,
									interpretation,
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

fn dehydrate_sub_value<V, I, D>(
	vocabulary: &mut V,
	interpretation: &mut I,
	layouts: &Layouts<I::Resource>,
	value: &Value,
	current_graph: Option<&I::Resource>,
	format: &ValueFormat<I::Resource>,
	env: &Environment<I::Resource>,
	output: &mut D,
) -> Result<(), Error>
where
	I: InterpretationMut<V>,
	I::Resource: Clone + Ord,
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
		vocabulary,
		interpretation,
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
			Self::Root(inputs) => inputs.get(i as usize).ok_or(i - inputs.len() as u32),
			Self::Child(parent, intros) => match parent.get(i) {
				Ok(r) => Ok(r),
				Err(j) => intros.get(j as usize).ok_or(j - intros.len() as u32),
			},
		}
	}

	#[must_use]
	pub fn bind<const N: usize>(&self, resources: [R; N]) -> Environment<R> {
		Environment::Child(self, resources.into_iter().collect())
	}

	#[must_use]
	pub fn intro<V, I>(
		&self,
		vocabulary: &mut V,
		interpretation: &mut I,
		count: u32,
	) -> Environment<R>
	where
		I: InterpretationMut<V, Resource = R>,
	{
		let mut intros = Vec::with_capacity(count as usize);
		for _ in 0..count {
			intros.push(interpretation.new_resource(vocabulary))
		}

		Environment::Child(self, intros)
	}
}

impl<'a, R: Clone> Environment<'a, R> {
	pub fn instantiate_pattern(&self, pattern: &Pattern<R>) -> Result<R, Error> {
		match pattern {
			Pattern::Var(x) => self
				.get(*x)
				.cloned()
				.map_err(|_| Error::UndeclaredVariable(*x)),
			Pattern::Resource(r) => Ok(r.clone()),
		}
	}

	pub fn instantiate_patterns(&self, patterns: &[Pattern<R>]) -> Result<Vec<R>, Error> {
		let mut result = Vec::with_capacity(patterns.len());

		for p in patterns {
			result.push(self.instantiate_pattern(p)?)
		}

		Ok(result)
	}

	pub fn instantiate_quad(
		&self,
		quad: Quad<&Pattern<R>, &Pattern<R>, &Pattern<R>, &Pattern<R>>,
	) -> Result<Quad<R, R, R, R>, Error> {
		Ok(Quad(
			self.instantiate_pattern(quad.0)?,
			self.instantiate_pattern(quad.1)?,
			self.instantiate_pattern(quad.2)?,
			quad.3.map(|g| self.instantiate_pattern(g)).transpose()?,
		))
	}

	pub fn instantiate_dataset<D>(
		&self,
		input: &BTreeDataset<Pattern<R>>,
		output: &mut D,
	) -> Result<(), Error>
	where
		D: grdf::MutableDataset<Subject = R, Predicate = R, Object = R, GraphLabel = R>,
	{
		for quad in input.quads() {
			output.insert(self.instantiate_quad(quad)?);
		}

		Ok(())
	}
}
