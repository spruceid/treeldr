use super::{
	Eval, EvalError, EvalTyped, RdfContext, RdfContextMut, ReverseScope, Scope, ScopeTypes,
};
use crate::{
	expr::{ExplicitList, ImplicitList, List, OrderedList, UnorderedList},
	TypeRef, Value,
};
use rdf_types::{
	dataset::{DatasetMut, PatternMatchingDataset},
	Quad,
};
use std::borrow::Cow;

impl<R: Clone + Ord> EvalTyped<R> for List<R> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		type_: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Explicit(list) => list.eval_typed(rdf, dataset, scope, type_),
			Self::Implicit(list) => list.eval_typed(rdf, dataset, scope, type_),
		}
	}

	fn eval_inverse_typed(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		type_: &TypeRef<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match self {
			Self::Explicit(list) => list.eval_inverse_typed(rdf, dataset, scope, type_, output),
			Self::Implicit(list) => list.eval_inverse_typed(rdf, dataset, scope, type_, output),
		}
	}
}

impl<R: Clone + Ord> EvalTyped<R> for ExplicitList<R> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		_type_: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut items = Vec::with_capacity(self.items.len());

		for e in &self.items {
			items.push(e.eval(rdf, dataset, scope)?);
		}

		Ok(Value::List(items))
	}

	fn eval_inverse_typed(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		_type_: &TypeRef<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match &output {
			Value::List(list) => {
				for (expr, item) in self.items.iter().zip(list) {
					expr.eval_inverse(rdf, dataset, scope, item)?;
				}

				Ok(())
			}
			_ => Err(EvalError::InvalidType),
		}
	}
}

impl<R: Clone + Ord> EvalTyped<R> for ImplicitList<R> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		type_: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Ordered(list) => list.eval_typed(rdf, dataset, scope, type_),
			Self::Unordered(list) => list.eval_typed(rdf, dataset, scope, type_),
		}
	}

	fn eval_inverse_typed(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		type_: &TypeRef<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match self {
			Self::Ordered(list) => list.eval_inverse_typed(rdf, dataset, scope, type_, output),
			Self::Unordered(list) => list.eval_inverse_typed(rdf, dataset, scope, type_, output),
		}
	}
}

impl<R: Clone + Ord> EvalTyped<R> for UnorderedList<R> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		type_ref: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		let item_ty = type_ref.as_list().unwrap().as_uniform().unwrap();

		let mut result = Vec::new();

		for values in self.bound.find_all(dataset, scope) {
			let scope = Scope::new(
				Some(scope),
				None,
				ScopeTypes::Repeat(item_ty, values.len() as u32),
				Cow::Owned(values),
			);

			result.push(self.body.eval(rdf, dataset, &scope)?);
		}

		Ok(Value::List(result))
	}

	fn eval_inverse_typed(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		_type_ref: &TypeRef<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match output {
			Value::List(list) => {
				for item in list {
					self.bound.inverse_once(rdf, scope, |rdf, scope| {
						self.body
							.eval_inverse(rdf, dataset, scope, item)
					})?;
				}

				Ok(())
			}
			_ => Err(EvalError::InvalidType),
		}
	}
}

impl<R: Clone + Ord> EvalTyped<R> for OrderedList<R> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		_type_ref: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		let mut result = Vec::new();

		let mut head = self
			.head
			.eval(rdf, dataset, scope)?
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
					return Err(EvalError::Empty);
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
					return Err(EvalError::Empty);
				};

				if rest_candidates.next().is_some() {
					return Err(EvalError::Ambiguity);
				}

				(first.clone(), rest.clone())
			};

			let scope = self
				.bound
				.find_one_with(dataset, &scope, |_| Some(first.clone()))?
				.ok_or(EvalError::Empty)?;

			result.push(self.body.eval(rdf, dataset, &scope)?);

			head = rest;
		}

		Ok(Value::List(result))
	}

	fn eval_inverse_typed(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		_type_ref: &TypeRef<R>,
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
						.inverse_once(rdf, scope, |rdf, scope| {
							self.body
								.eval_inverse(rdf, dataset, scope, item)
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

				Ok(())
			}
			_ => Err(EvalError::InvalidType),
		}
	}
}
