use super::{
	Eval, EvalError, EvalTyped, MissingData, RdfContext, RdfContextMut, Scope, ScopeTypes,
	ScopeValues,
};
use crate::{
	expr::{ExplicitList, ImplicitList, List, OrderedList, UnorderedList},
	rdf::DebugSubstitution,
	Registry, TypeRef, Value,
};
use rdf_types::{
	dataset::{DatasetMut, PatternMatchingDataset},
	Quad,
};

impl<P: Copy + Ord, R: Clone + Ord> EvalTyped<P, R> for List<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl RdfContext<Resource = R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<P, R>,
		type_: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Explicit(list) => list.eval_typed(registry, rdf, dataset, scope, type_),
			Self::Implicit(list) => list.eval_typed(registry, rdf, dataset, scope, type_),
		}
	}

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut Scope<P, R>,
		type_: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match self {
			Self::Explicit(list) => {
				list.eval_inverse_typed(registry, rdf, dataset, scope, type_, output)
			}
			Self::Implicit(list) => {
				list.eval_inverse_typed(registry, rdf, dataset, scope, type_, output)
			}
		}
	}
}

impl<P: Copy + Ord, R: Clone + Ord> EvalTyped<P, R> for ExplicitList<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl RdfContext<Resource = R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<P, R>,
		_type_: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut items = Vec::with_capacity(self.items.len());

		for e in &self.items {
			items.push(e.eval(registry, rdf, dataset, scope)?);
		}

		Ok(Value::List(items))
	}

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut Scope<P, R>,
		_type_: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match &output {
			Value::List(list) => {
				for (expr, item) in self.items.iter().zip(list) {
					expr.eval_inverse(registry, rdf, dataset, scope, item)?;
				}

				Ok(())
			}
			_ => Err(EvalError::InvalidType),
		}
	}
}

impl<P: Copy + Ord, R: Clone + Ord> EvalTyped<P, R> for ImplicitList<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl RdfContext<Resource = R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<P, R>,
		type_: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Ordered(list) => list.eval_typed(registry, rdf, dataset, scope, type_),
			Self::Unordered(list) => list.eval_typed(registry, rdf, dataset, scope, type_),
		}
	}

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut Scope<P, R>,
		type_: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match self {
			Self::Ordered(list) => {
				list.eval_inverse_typed(registry, rdf, dataset, scope, type_, output)
			}
			Self::Unordered(list) => {
				list.eval_inverse_typed(registry, rdf, dataset, scope, type_, output)
			}
		}
	}
}

impl<P: Copy + Ord, R: Clone + Ord> EvalTyped<P, R> for UnorderedList<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl RdfContext<Resource = R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<P, R>,
		type_ref: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		let item_ty = registry
			.get_type(type_ref)
			.unwrap()
			.as_list()
			.unwrap()
			.as_uniform()
			.unwrap();

		let mut result = Vec::new();

		for values in self.bound.find_all(dataset, scope) {
			let scope = scope.begin(
				None,
				ScopeTypes::Repeat(item_ty, values.len() as u32),
				ScopeValues::ConstOwned(values),
			);

			result.push(self.body.eval(registry, rdf, dataset, &scope)?);
		}

		result.sort_unstable();

		Ok(Value::List(result))
	}

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut Scope<P, R>,
		_type_ref: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match output {
			Value::List(list) => {
				for item in list {
					self.bound.eval_inverse(
						registry,
						rdf,
						dataset,
						scope,
						|rdf, dataset, scope| {
							self.body.eval_inverse(registry, rdf, dataset, scope, item)
						},
					)?;
				}

				Ok(())
			}
			_ => Err(EvalError::InvalidType),
		}
	}
}

impl<P: Copy + Ord, R: Clone + Ord> EvalTyped<P, R> for OrderedList<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl RdfContext<Resource = R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<P, R>,
		_type_ref: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut result = Vec::new();

		let mut head = self
			.head
			.eval(registry, rdf, dataset, scope)?
			.into_resource()
			.unwrap();

		while head != self.components.nil {
			let (first, rest) = {
				let mut first_candidates = dataset.quad_pattern_matching(
					Quad(
						Some(&head),
						Some(&self.components.first),
						None,
						Some(scope.graph()),
					)
					.into(),
				);

				let Some(Quad(_, _, first, _)) = first_candidates.next() else {
					let mut substitution = DebugSubstitution::new();
					return Err(EvalError::MissingData(MissingData::Empty(
						[Quad(
							substitution.insert_resource(rdf, &head),
							substitution.insert_resource(rdf, &self.components.first),
							substitution.new_resource(),
							scope.graph().map(|g| substitution.insert_resource(rdf, g)),
						)]
						.into_iter()
						.collect(),
					)));
				};

				if first_candidates.next().is_some() {
					return Err(EvalError::Ambiguity);
				}

				let mut rest_candidates = dataset.quad_pattern_matching(
					Quad(
						Some(&head),
						Some(&self.components.rest),
						None,
						Some(scope.graph()),
					)
					.into(),
				);

				let Some(Quad(_, _, rest, _)) = rest_candidates.next() else {
					let mut substitution = DebugSubstitution::new();
					return Err(EvalError::MissingData(MissingData::Empty(
						[Quad(
							substitution.insert_resource(rdf, &head),
							substitution.insert_resource(rdf, &self.components.rest),
							substitution.new_resource(),
							scope.graph().map(|g| substitution.insert_resource(rdf, g)),
						)]
						.into_iter()
						.collect(),
					)));
				};

				if rest_candidates.next().is_some() {
					return Err(EvalError::Ambiguity);
				}

				(first.clone(), rest.clone())
			};

			let scope = self
				.bound
				.find_one_with(dataset, scope, |_| Ok(Some(Value::Resource(first.clone()))))?
				.ok_or_else(|| {
					let mut substitution = DebugSubstitution::new();
					EvalError::MissingData(MissingData::Empty(scope.substitute_dataset(
						rdf,
						&mut substitution,
						&self.bound.dataset,
					)))
				})?;

			result.push(self.body.eval(registry, rdf, dataset, &scope)?);

			head = rest;
		}

		Ok(Value::List(result))
	}

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut Scope<P, R>,
		_type_ref: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		assert_eq!(self.bound.intro.len(), 1);

		match output {
			Value::List(list) => {
				let mut rest = self.components.nil.clone();

				for item in list.iter().rev() {
					let head = rdf.new_resource();

					let first = self
						.bound
						.find_one_inverse(rdf, dataset, scope, |rdf, dataset, scope| {
							self.body.eval_inverse(registry, rdf, dataset, scope, item)
						})?
						.into_iter()
						.next()
						.unwrap()
						.into_resource()
						.unwrap();

					dataset.insert(Quad(
						head.clone(),
						self.components.first.clone(),
						first,
						scope.graph().cloned(),
					));
					dataset.insert(Quad(
						head.clone(),
						self.components.rest.clone(),
						rest,
						scope.graph().cloned(),
					));
					rest = head;
				}

				self.head
					.eval_inverse(registry, rdf, dataset, scope, &Value::Resource(rest))
			}
			_ => Err(EvalError::InvalidType),
		}
	}
}
