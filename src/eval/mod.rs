mod scope;

use iref::IriBuf;
use rdf_types::dataset::BTreeDataset;
pub use scope::*;

use crate::{
	expr::{Expr, InnerExpr},
	rdf::{DebugSubstitution, EvalDataset, RdfContextMut},
	Registry, TypeRef, Value,
};

mod bound;
mod call;
mod list;
mod literal;
mod map;
mod r#match;
mod pattern;

#[derive(Debug, thiserror::Error)]
pub enum MissingData {
	#[error("empty selection")]
	Empty(BTreeDataset),

	#[error("no variant match")]
	NotVariantMatch,

	#[error("no matching IRI representation")]
	NoMatchingIriRepresentation,

	#[error("no matching literal representation")]
	NoMatchingLiteralRepresentation,
}

#[derive(Debug, thiserror::Error)]
pub enum EvalError<R = rdf_types::Term> {
	#[error(transparent)]
	MissingData(MissingData),

	#[error("ambiguity")]
	Ambiguity,

	#[error("invalid type")]
	InvalidType,

	#[error("invalid value")]
	InvalidValue,

	#[error("unknown key")]
	UnknownKey(Value<R>),

	#[error("unknown variant `{0}`")]
	UnknownVariant(String),

	#[error("invalid number of arguments (expected {expected}, found {found})")]
	ArgumentCount { expected: u32, found: u32 },

	#[error("ambiguous IRI representation")]
	AmbiguousIriRepresentation(IriBuf, IriBuf),

	#[error("ambiguous literal representation")]
	AmbiguousLiteralRepresentation(String, String),

	#[error("resource unification error")]
	ResourceUnificationError,
}

pub trait Eval<P, R: Clone> {
	fn eval(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &Scope<P, R>,
	) -> Result<Value<R>, EvalError<R>>;

	fn eval_inverse(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &mut Scope<P, R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>>;
}

pub trait EvalTyped<P, R: Clone> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &Scope<P, R>,
		r#type: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>>;

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &mut Scope<P, R>,
		r#type: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>>;
}

impl<P: Copy + Ord, R: Ord + Clone> Eval<P, R> for Expr<P, R> {
	fn eval(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &Scope<P, R>,
	) -> Result<Value<R>, EvalError<R>> {
		self.inner
			.eval_typed(registry, rdf, dataset, scope, self.r#type)
	}

	fn eval_inverse(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &mut Scope<P, R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		self.inner
			.eval_inverse_typed(registry, rdf, dataset, scope, self.r#type, output)
	}
}

impl<P: Copy + Ord, R: Ord + Clone> EvalTyped<P, R> for InnerExpr<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &Scope<P, R>,
		r#type: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Let(bindings, e) => {
				let sub_scope = bindings
					.eval(registry, rdf, dataset, scope)?
					.ok_or_else(|| {
						let mut substitution = DebugSubstitution::new();
						EvalError::MissingData(MissingData::Empty(scope.substitute_dataset(
							rdf,
							&mut substitution,
							&bindings.dataset,
						)))
					})?;
				e.eval(registry, rdf, dataset, &sub_scope)
			}
			Self::Var(i) => Ok(scope.get(*i).unwrap().clone()),
			Self::Resource(r) => Ok(Value::Resource(r.clone())),
			Self::Literal(l) => l.eval_typed(registry, rdf, dataset, scope, r#type),
			Self::List(list) => list.eval_typed(registry, rdf, dataset, scope, r#type),
			Self::Map(map) => map.eval_typed(registry, rdf, dataset, scope, r#type),
			Self::Match(m) => m.eval_typed(registry, rdf, dataset, scope, r#type),
			Self::Call(call) => call.eval(registry, rdf, dataset, scope),
		}
	}

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		scope: &mut Scope<P, R>,
		r#type: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match self {
			Self::Let(bindings, e) => {
				bindings.eval_inverse(registry, rdf, dataset, scope, |rdf, dataset, scope| {
					e.eval_inverse(registry, rdf, dataset, scope, output)
				})
			}
			Self::Var(i) => scope.set(rdf, *i, output.clone()),
			Self::Resource(a) => match output {
				Value::Resource(b) => rdf
					.unify(a, b)
					.map_err(|_| EvalError::ResourceUnificationError),
				_ => Err(EvalError::InvalidType),
			},
			Self::Literal(l) => l.eval_inverse_typed(registry, rdf, dataset, scope, r#type, output),
			Self::List(list) => {
				list.eval_inverse_typed(registry, rdf, dataset, scope, r#type, output)
			}
			Self::Map(map) => map.eval_inverse_typed(registry, rdf, dataset, scope, r#type, output),
			Self::Match(m) => m.eval_inverse_typed(registry, rdf, dataset, scope, r#type, output),
			Self::Call(call) => call.eval_inverse(registry, rdf, dataset, scope, output),
		}
	}
}
