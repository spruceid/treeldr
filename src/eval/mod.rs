mod scope;

use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};
pub use scope::*;

use crate::{
	expr::{Expr, InnerExpr},
	TypeRef, Value,
};

mod rdf;
pub use rdf::*;

mod bound;
mod call;
mod list;
mod literal;
mod map;
mod r#match;

#[derive(Debug, thiserror::Error)]
pub enum EvalError<R> {
	#[error("ambiguity")]
	Ambiguity,

	#[error("empty selection")]
	Empty,

	#[error("invalid type")]
	InvalidType,

	#[error("invalid value")]
	InvalidValue,

	#[error("unknown key")]
	UnknownKey(Value<R>),

	#[error("unknown variant `{0}`")]
	UnknownVariant(String),
}

pub trait Eval<R: Clone> {
	fn eval(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
	) -> Result<Value<R>, EvalError<R>>;

	fn eval_inverse(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>>;
}

pub trait EvalTyped<R: Clone> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		type_: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>>;

	fn eval_inverse_typed(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		type_: &TypeRef<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>>;
}

impl<R: Ord + Clone> Eval<R> for Expr<R> {
	fn eval(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
	) -> Result<Value<R>, EvalError<R>> {
		let scope = self
			.bound
			.find_one(dataset, scope)?
			.ok_or(EvalError::Empty)?;

		self.inner.eval_typed(rdf, dataset, &scope, &self.type_)
	}

	fn eval_inverse(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		scope: &mut ReverseScope<R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		self.bound.inverse_once(rdf, scope, |rdf, scope| {
			self.inner
				.eval_inverse_typed(rdf, dataset, scope, &self.type_, output)
		})?;
		Ok(())
	}
}

impl<R: Ord + Clone> EvalTyped<R> for InnerExpr<R> {
	fn eval_typed(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
		type_: &TypeRef<R>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Var(i) => Ok(scope.get(*i).unwrap().clone()),
			Self::Literal(l) => l.eval_typed(rdf, dataset, scope, type_),
			Self::List(list) => list.eval_typed(rdf, dataset, scope, type_),
			Self::Map(map) => map.eval_typed(rdf, dataset, scope, type_),
			Self::Match(m) => m.eval_typed(rdf, dataset, scope, type_),
			Self::Call(call) => call.eval(rdf, dataset, scope),
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
			Self::Var(i) => scope.set(*i, output.clone()),
			Self::Literal(l) => l.eval_inverse_typed(rdf, dataset, scope, type_, output),
			Self::List(list) => list.eval_inverse_typed(rdf, dataset, scope, type_, output),
			Self::Map(map) => map.eval_inverse_typed(rdf, dataset, scope, type_, output),
			Self::Match(m) => m.eval_inverse_typed(rdf, dataset, scope, type_, output),
			Self::Call(call) => call.eval_inverse(rdf, dataset, scope, output),
		}
	}
}
