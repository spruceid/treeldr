use rdf_types::{dataset::DatasetMut, Quad};

use super::EvalError;
use crate::{pattern::Substitution, rdf::RdfContextMut, TermPattern, TypeRef, Value};
use std::borrow::Cow;

#[derive(Clone, Copy)]
pub enum ScopeTypes<'a> {
	Slice(&'a [TypeRef]),
	Repeat(TypeRef, u32),
}

impl<'a> ScopeTypes<'a> {
	pub fn get(&self, i: u32) -> Option<TypeRef> {
		match self {
			Self::Slice(slice) => slice.get(i as usize).copied(),
			Self::Repeat(ty, count) => {
				if i < *count {
					Some(*ty)
				} else {
					None
				}
			}
		}
	}
}

#[derive(Clone)]
pub struct Scope<'parent, R: Clone> {
	parent: Option<(&'parent Self, u32)>,
	graph: Option<R>,
	types: ScopeTypes<'parent>,
	values: Cow<'parent, [Value<R>]>,
}

impl<'parent, R: Clone> Scope<'parent, R> {
	/// Create a new scope with the given parent, graph, and newly bound values.
	///
	/// If graph is `None` then is will inherit the graph is the parent scope.
	/// If the parent graph is `None`, the inherited graph will be `None`.
	pub fn new(
		parent: Option<&'parent Self>,
		graph: Option<Option<R>>,
		types: ScopeTypes<'parent>,
		values: Cow<'parent, [Value<R>]>,
	) -> Self {
		Self {
			parent: parent.map(|p| (p, p.len())),
			graph: graph.unwrap_or_else(|| parent.map(|p| p.graph.clone()).unwrap_or_default()),
			types,
			values,
		}
	}

	pub fn top_values(&self) -> &[Value<R>] {
		&self.values
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn len(&self) -> u32 {
		match self.parent {
			Some((_, parent_len)) => parent_len + self.values.len() as u32,
			None => self.values.len() as u32,
		}
	}

	pub fn graph(&self) -> Option<&R> {
		self.graph.as_ref()
	}

	pub fn type_of(&self, i: u32) -> Option<TypeRef> {
		match self.parent {
			Some((parent, parent_len)) => {
				if i < parent_len {
					parent.type_of(i)
				} else {
					self.types.get(i - parent_len)
				}
			}
			None => self.types.get(i),
		}
	}

	pub fn get(&self, i: u32) -> Option<&Value<R>> {
		match self.parent {
			Some((parent, parent_len)) => {
				if i < parent_len {
					parent.get(i)
				} else {
					self.values.get((i - parent_len) as usize)
				}
			}
			None => self.values.get(i as usize),
		}
	}
}

impl<'a, R: Clone> Default for Scope<'a, R> {
	fn default() -> Self {
		Self {
			parent: None,
			graph: None,
			types: ScopeTypes::Slice(&[]),
			values: Cow::Borrowed(&[]),
		}
	}
}

pub struct ReverseScope<'parent, R> {
	parent: Option<(&'parent Self, u32)>,
	graph: Option<R>,
	values: Substitution<R>,
	dataset: Vec<Quad<TermPattern<R>>>,
}

impl<'a, R> ReverseScope<'a, R> {
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn len(&self) -> u32 {
		match self.parent {
			Some((_, parent_len)) => parent_len + self.values.len() as u32,
			None => self.values.len() as u32,
		}
	}

	pub fn graph(&self) -> Option<&R> {
		self.graph.as_ref()
	}

	pub fn get(&self, i: u32) -> Option<&Value<R>> {
		match self.parent {
			Some((parent, parent_len)) => {
				if i < parent_len {
					parent.get(i)
				} else {
					self.values.get(i - parent_len)
				}
			}
			None => self.values.get(i),
		}
	}

	pub fn get_or_insert(&self, i: u32, f: impl FnOnce() -> Value<R>) -> Option<&Value<R>> {
		match self.parent {
			Some((parent, parent_len)) => {
				if i < parent_len {
					parent.get(i)
				} else {
					Some(self.values.get_or_insert(i - parent_len, f))
				}
			}
			None => Some(self.values.get_or_insert(i, f)),
		}
	}

	pub fn set(&self, i: u32, value: Value<R>) -> Result<(), EvalError<R>>
	where
		R: Eq,
	{
		match self.parent {
			Some((parent, parent_len)) => {
				if i < parent_len {
					parent.set(i, value)
				} else {
					let j = i - parent_len;
					self.values.set(j, value).map_err(|_| EvalError::Ambiguity)
				}
			}
			None => self.values.set(i, value).map_err(|_| EvalError::Ambiguity),
		}
	}

	pub fn extend_dataset(&mut self, quads: impl Iterator<Item = Quad<TermPattern<R>>>) {
		self.dataset.extend(quads);
	}
}

impl<'a, R: Clone> ReverseScope<'a, R> {
	pub fn root(graph: Option<R>) -> Self {
		Self::new(graph, &[])
	}

	pub fn new(graph: Option<R>, types: &[TypeRef]) -> Self {
		Self {
			parent: None,
			graph,
			values: Substitution::new(types.len() as u32, |_| None),
			dataset: Vec::new(),
		}
	}

	/// Creates a sub scope that extends this scope.
	///
	/// This scope will be the parent scope of the created sub scope.
	/// The sub scope is then passed to the function `f`.
	///
	/// This guarantees that the sub scopes does not outlive the parent scope
	/// and that is has exclusive access over the parent scope.
	pub fn begin<C, D>(
		&mut self,
		rdf: &mut C,
		dataset: &mut D,
		graph: Option<Option<R>>,
		types: &[TypeRef],
		f: impl FnOnce(&mut C, &mut D, &mut ReverseScope<R>) -> Result<(), EvalError<R>>,
	) -> Result<Vec<Value<R>>, EvalError<R>>
	where
		C: RdfContextMut<R>,
		D: DatasetMut<Resource = R>,
	{
		let (substitution, more_patterns) = {
			let mut sub_scope = ReverseScope::<R> {
				parent: Some((self, self.len())),
				graph: graph.unwrap_or_else(|| self.graph.clone()),
				values: Substitution::new(types.len() as u32, |_| None),
				dataset: Vec::new(),
			};

			f(rdf, dataset, &mut sub_scope)?;

			sub_scope.end(rdf, dataset)?
		};

		self.dataset.extend(more_patterns);
		Ok(substitution)
	}

	fn end(
		self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
	) -> Result<(Vec<Value<R>>, Vec<Quad<TermPattern<R>>>), EvalError<R>> {
		let mut patterns = Vec::new();
		for quad_pattern in &self.dataset {
			match TermPattern::quad_into_resources(self.realize_quad(rdf, quad_pattern)?) {
				Ok(quad) => {
					dataset.insert(quad);
				}
				Err(quad_pattern) => patterns.push(quad_pattern),
			}
		}

		let substitution = self
			.values
			.try_into_total_with(|_| Ok(Value::Resource(rdf.new_resource())))?;

		Ok((substitution, patterns))
	}

	fn realize_quad(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		quad: &Quad<TermPattern<R>>,
	) -> Result<Quad<TermPattern<R>>, EvalError<R>> {
		Ok(Quad(
			self.realize_term(rdf, &quad.0)?,
			self.realize_term(rdf, &quad.1)?,
			self.realize_term(rdf, &quad.2)?,
			quad.3
				.as_ref()
				.map(|g| self.realize_term(rdf, g))
				.transpose()?,
		))
	}

	fn realize_term(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		t: &TermPattern<R>,
	) -> Result<TermPattern<R>, EvalError<R>> {
		match t {
			TermPattern::Var(x) => {
				match self.get_or_insert(*x, || Value::Resource(rdf.new_resource())) {
					Some(Value::Resource(r)) => Ok(TermPattern::Resource(r.clone())),
					Some(_) => Err(EvalError::InvalidType),
					None => Ok(TermPattern::Var(*x)),
				}
			}
			TermPattern::Resource(r) => Ok(TermPattern::Resource(r.clone())),
		}
	}
}
