use super::EvalError;
use crate::{
	pattern::Substitution,
	rdf::{DebugSubstitution, RdfContext, RdfContextMut},
	value::List,
	DatasetPattern, TermPattern, TypeRef, Value,
};
use rdf_types::{
	dataset::{BTreeDataset, DatasetMut},
	Quad, Term,
};

#[derive(Clone, Copy)]
pub enum ScopeTypes<'a, P> {
	Slice(&'a [TypeRef<P>]),
	TwoSlices(&'a [TypeRef<P>], &'a [TypeRef<P>]),
	// Bindings(&'a [Binding<P, R>]),
	Repeat(TypeRef<P>, u32),
}

impl<'a, P> ScopeTypes<'a, P> {
	pub fn len(&self) -> u32 {
		match self {
			Self::Slice(s) => s.len() as u32,
			Self::TwoSlices(a, b) => (a.len() + b.len()) as u32,
			// Self::Bindings(b) => b.len() as u32,
			Self::Repeat(_, len) => *len,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn get(&self, i: u32) -> Option<TypeRef<P>>
	where
		P: Copy,
	{
		match self {
			Self::Slice(slice) => slice.get(i as usize).copied(),
			Self::TwoSlices(a, b) => {
				if i < a.len() as u32 {
					a.get(i as usize).copied()
				} else {
					b.get(i as usize - a.len()).copied()
				}
			}
			// Self::Bindings(b) => b.get(i as usize).map(|b| b.r#type),
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
pub struct Scope<'parent, P, R> {
	parent: Option<(&'parent Self, u32)>,
	graph: Option<R>,
	types: ScopeTypes<'parent, P>,
	values: ScopeValues<'parent, R>,
	dataset: Vec<Quad<TermPattern<R>>>,
}

impl<'parent, P, R> Scope<'parent, P, R> {
	pub fn new(
		graph: Option<R>,
		types: ScopeTypes<'parent, P>,
		values: ScopeValues<'parent, R>,
	) -> Self {
		Self {
			parent: None,
			graph,
			types,
			values,
			dataset: Vec::new(),
		}
	}

	pub fn root(graph: Option<R>) -> Self {
		Self::new(
			graph,
			ScopeTypes::Slice(&[]),
			ScopeValues::ConstOwned(Vec::new()),
		)
	}

	pub fn parent(&self) -> Option<&Scope<P, R>> {
		self.parent.map(|(p, _)| p)
	}

	pub fn top_const_values(&self) -> &[Value<R>] {
		self.values.as_const().unwrap()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn len(&self) -> u32 {
		match self.parent {
			Some((_, parent_len)) => parent_len + self.values.len(),
			None => self.values.len(),
		}
	}

	pub fn graph(&self) -> Option<&R> {
		self.graph.as_ref()
	}

	pub fn type_of(&self, i: u32) -> Option<TypeRef<P>>
	where
		P: Copy,
	{
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

	pub fn set(
		&self,
		rdf: &mut impl RdfContextMut<Resource = R>,
		i: u32,
		value: Value<R>,
	) -> Result<(), EvalError<R>>
	where
		R: Eq,
	{
		match self.parent {
			Some((parent, parent_len)) => {
				if i < parent_len {
					parent.set(rdf, i, value)
				} else {
					let j = i - parent_len;
					self.values
						.set(rdf, j, value)
						.map_err(|_| EvalError::Ambiguity)
				}
			}
			None => self
				.values
				.set(rdf, i, value)
				.map_err(|_| EvalError::Ambiguity),
		}
	}

	pub fn extend_dataset(&mut self, quads: impl Iterator<Item = Quad<TermPattern<R>>>) {
		self.dataset.extend(quads);
	}

	pub fn substitute_pattern(
		&self,
		rdf: &impl RdfContext<Resource = R>,
		substitution: &mut DebugSubstitution<R>,
		pattern: &TermPattern<R>,
	) -> Term
	where
		R: Ord + Clone,
	{
		match pattern {
			TermPattern::Resource(r) => substitution.insert_resource(rdf, r),
			TermPattern::Var(x) => match self.get(*x) {
				Some(Value::Resource(r)) => substitution.insert_resource(rdf, r),
				Some(_) => panic!("not a resource"),
				None => substitution.insert_var(*x),
			},
		}
	}

	pub fn substitute_dataset(
		&self,
		rdf: &impl RdfContext<Resource = R>,
		substitution: &mut DebugSubstitution<R>,
		dataset: &DatasetPattern<R>,
	) -> BTreeDataset
	where
		R: Clone + Ord,
	{
		dataset
			.iter()
			.map(|q| q.map(|pattern| self.substitute_pattern(rdf, substitution, pattern)))
			.collect()
	}
}

impl<'a, P, R: Clone> Default for Scope<'a, P, R> {
	fn default() -> Self {
		Self {
			parent: None,
			graph: None,
			types: ScopeTypes::Slice(&[]),
			values: ScopeValues::ConstBorrowed(&[]),
			dataset: Vec::new(),
		}
	}
}

impl<'a, P, R: Clone> Scope<'a, P, R> {
	pub fn begin(
		&self,
		graph: Option<Option<R>>,
		types: ScopeTypes<'a, P>,
		values: ScopeValues<'a, R>,
	) -> Scope<P, R> {
		Scope {
			parent: Some((self, self.len())),
			graph: graph.unwrap_or_else(|| self.graph.clone()),
			types,
			values,
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
	pub fn begin_mut<C, D>(
		&mut self,
		rdf: &mut C,
		dataset: &mut D,
		graph: Option<Option<R>>,
		// types: &[TypeRef],
		types: ScopeTypes<P>,
		f: impl FnOnce(&mut C, &mut D, &mut Scope<P, R>) -> Result<(), EvalError<R>>,
	) -> Result<Vec<Value<R>>, EvalError<R>>
	where
		C: RdfContextMut<Resource = R>,
		D: DatasetMut<Resource = R>,
	{
		let types_len = types.len();
		let (substitution, more_patterns) = {
			let mut sub_scope = Scope::<P, R> {
				parent: Some((self, self.len())),
				graph: graph.unwrap_or_else(|| self.graph.clone()),
				// types: ScopeTypes::Slice(types),
				types,
				values: ScopeValues::Mut(Substitution::new(types_len, |_| None)),
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
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl DatasetMut<Resource = R>,
	) -> Result<(List<R>, UnresolvedQuads<R>), EvalError<R>> {
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
			.into_mut()
			.unwrap()
			.try_into_total_with(|_| Ok(Value::Resource(rdf.new_resource())))?;

		Ok((substitution, patterns))
	}

	fn realize_quad(
		&self,
		rdf: &mut impl RdfContextMut<Resource = R>,
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
		rdf: &mut impl RdfContextMut<Resource = R>,
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

pub type UnresolvedQuads<R> = Vec<Quad<TermPattern<R>>>;

#[derive(Clone)]
pub enum ScopeValues<'a, R> {
	ConstBorrowed(&'a [Value<R>]),
	ConstOwned(Vec<Value<R>>),
	Mut(Substitution<R>),
}

impl<'a, R> ScopeValues<'a, R> {
	fn len(&self) -> u32 {
		match self {
			Self::ConstBorrowed(values) => values.len() as u32,
			Self::ConstOwned(values) => values.len() as u32,
			Self::Mut(values) => values.len(),
		}
	}

	fn as_const(&self) -> Option<&[Value<R>]> {
		match self {
			Self::ConstBorrowed(values) => Some(values),
			Self::ConstOwned(values) => Some(values),
			Self::Mut(_) => None,
		}
	}

	fn into_mut(self) -> Option<Substitution<R>> {
		match self {
			Self::Mut(values) => Some(values),
			_ => None,
		}
	}

	fn get(&self, i: u32) -> Option<&Value<R>> {
		match self {
			Self::ConstBorrowed(values) => values.get(i as usize),
			Self::ConstOwned(values) => values.get(i as usize),
			Self::Mut(values) => values.get(i),
		}
	}

	pub fn get_or_insert(&self, i: u32, f: impl FnOnce() -> Value<R>) -> &Value<R> {
		match self {
			Self::Mut(values) => values.get_or_insert(i, f),
			_ => panic!("cannot mutate const scope"),
		}
	}

	pub fn set(
		&self,
		rdf: &mut impl RdfContextMut<Resource = R>,
		i: u32,
		value: Value<R>,
	) -> Result<(), EvalError<R>>
	where
		R: Eq,
	{
		match self {
			Self::Mut(values) => match values.set(i, value) {
				Ok(()) => Ok(()),
				Err((Value::Resource(a), Value::Resource(b))) => rdf
					.unify(a, &b)
					.map_err(|_| EvalError::ResourceUnificationError),
				Err(_) => Err(EvalError::Ambiguity),
			},
			_ => panic!("cannot mutate const scope"),
		}
	}
}
