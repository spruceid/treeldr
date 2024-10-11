use super::{EvalError, RdfContextMut};
use crate::{pattern::Substitution, TypeRef, Value};
use std::borrow::Cow;

#[derive(Clone, Copy)]
pub enum ScopeTypes<'a, R> {
	Slice(&'a [TypeRef<R>]),
	Repeat(&'a TypeRef<R>, u32),
}

impl<'a, R> ScopeTypes<'a, R> {
	pub fn get(&self, i: u32) -> Option<&'a TypeRef<R>> {
		match self {
			Self::Slice(slice) => slice.get(i as usize),
			Self::Repeat(ty, count) => {
				if i < *count {
					Some(ty)
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
	types: ScopeTypes<'parent, R>,
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
		types: ScopeTypes<'parent, R>,
		values: Cow<'parent, [Value<R>]>,
	) -> Self {
		Self {
			parent: parent.map(|p| (p, p.len())),
			graph: graph.unwrap_or_else(|| parent.map(|p| p.graph.clone()).unwrap_or_default()),
			types,
			values,
		}
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

	pub fn type_of(&self, i: u32) -> Option<&TypeRef<R>> {
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
}

impl<'a, R> ReverseScope<'a, R> {
	pub fn new(graph: Option<R>, types: &'a [TypeRef<R>]) -> Self
	where
		R: Clone,
	{
		Self {
			parent: None,
			graph,
			values: Substitution::new(types.len() as u32, |_| None),
		}
	}

	/// Creates a sub scope that extends this scope.
	///
	/// This scope will be the parent scope of the created sub scope.
	/// The sub scope is then passed to the function `f`.
	///
	/// This guarantees that the sub scopes does not outlive the parent scope
	/// and that is has exclusive access over the parent scope.
	pub fn begin<'b, C>(
		&'b mut self,
		rdf: &mut C,
		graph: Option<Option<R>>,
		types: &'b [TypeRef<R>],
		f: impl FnOnce(&mut C, &mut ReverseScope<'b, R>) -> Result<(), EvalError<R>>,
	) -> Result<Vec<Value<R>>, EvalError<R>>
	where
		C: RdfContextMut<R>,
		R: Clone,
	{
		let mut sub_scope = ReverseScope::<'b, R> {
			parent: Some((self, self.len())),
			graph: graph.unwrap_or_else(|| self.graph.clone()),
			values: Substitution::new(types.len() as u32, |_| None),
		};

		f(rdf, &mut sub_scope)?;
		sub_scope.end(rdf)
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

	pub fn end(self, rdf: &mut impl RdfContextMut<R>) -> Result<Vec<Value<R>>, EvalError<R>> {
		self.values
			.try_into_total_with(|_| Ok(Value::Resource(rdf.new_resource())))
	}
}
