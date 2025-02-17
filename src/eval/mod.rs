mod scope;

use std::{borrow::Cow, hash::Hash};

use iref::{Iri, IriBuf};
use rdf_types::{
	dataset::{BTreeDataset, MultiPatternMatchingDataset},
	interpretation::{ConstGenerativeInterpretation, ReverseInterpretation},
	pattern::CanonicalQuadPattern,
	CowLiteral, CowTerm, Quad,
};
pub use scope::*;

use crate::{
	expr::{Expr, InnerExpr},
	rdf::{ContextStack, DebugSubstitution, ResourceUnificationFailed},
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
pub enum EvalError<R = rdf_types::LocalTerm> {
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

impl<R> From<ResourceUnificationFailed> for EvalError<R> {
	fn from(_: ResourceUnificationFailed) -> Self {
		Self::ResourceUnificationError
	}
}

pub trait EvalContext {
	type Resource;

	type QuadPatternMatching<'a>: Iterator<Item = Quad<&'a Self::Resource>>
	where
		Self: 'a,
		Self::Resource: 'a;

	type Iris<'a>: Iterator<Item = Cow<'a, Iri>>
	where
		Self: 'a;

	type Literals<'a>: Iterator<Item = CowLiteral<'a>>
	where
		Self: 'a;

	type SubContext<'a>: EvalContext<Resource = Self::Resource>
	where
		Self: 'a;

	fn pattern_matching<'a>(
		&'a self,
		pattern: CanonicalQuadPattern<&'a Self::Resource>,
	) -> Self::QuadPatternMatching<'a>;

	fn quad_objects<'a>(
		&'a self,
		graph: Option<&'a Self::Resource>,
		subject: &'a Self::Resource,
		predicate: &'a Self::Resource,
	) -> QuadObjects<'a, Self> {
		QuadObjects(
			self.pattern_matching(Quad(Some(subject), Some(predicate), None, Some(graph)).into()),
		)
	}

	fn iri_interpretation(&self, i: &Iri) -> Option<Self::Resource>;

	fn iris_of<'a>(&'a self, r: &'a Self::Resource) -> Self::Iris<'a>;

	fn literals_of<'a>(&'a self, r: &'a Self::Resource) -> Self::Literals<'a>;

	fn terms_of<'a>(&'a self, r: &'a Self::Resource) -> TermsOf<'a, Self> {
		TermsOf {
			iris: self.iris_of(r),
			literals: self.literals_of(r),
		}
	}

	fn new_resource(&mut self) -> Self::Resource;

	fn insert_iri<'a>(&mut self, i: impl Into<Cow<'a, Iri>>) -> Self::Resource;

	fn insert_literal<'a>(&mut self, i: impl Into<CowLiteral<'a>>) -> Self::Resource;

	fn insert(&mut self, quad: Quad<Self::Resource>);

	fn unify(&mut self, a: &Self::Resource, b: &Self::Resource);

	fn begin(&self) -> Self::SubContext<'_>;
}

pub struct TermsOf<'a, C: 'a + ?Sized + EvalContext> {
	iris: C::Iris<'a>,
	literals: C::Literals<'a>,
}

impl<'a, C: 'a + ?Sized + EvalContext> Iterator for TermsOf<'a, C> {
	type Item = CowTerm<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.iris
			.next()
			.map(CowTerm::Iri)
			.or_else(|| self.literals.next().map(CowTerm::Literal))
	}
}

pub struct QuadObjects<'a, C: 'a + ?Sized + EvalContext>(C::QuadPatternMatching<'a>);

impl<'a, C> Iterator for QuadObjects<'a, C>
where
	C: 'a + ?Sized + EvalContext,
{
	type Item = &'a C::Resource;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(Quad::into_object)
	}
}

impl<'a, I, D> EvalContext for ContextStack<'a, I, D>
where
	I: ?Sized + ConstGenerativeInterpretation<Resource: Clone + Ord + Hash> + ReverseInterpretation,
	D: ?Sized + MultiPatternMatchingDataset<Resource = I::Resource>,
{
	type Resource = I::Resource;

	type QuadPatternMatching<'b>
		= crate::rdf::QuadPatternMatching<'b, 'b, D>
	where
		Self: 'b,
		Self::Resource: 'b;

	type Iris<'b>
		= crate::rdf::IrisOf<'b, I>
	where
		Self: 'b;

	type Literals<'b>
		= crate::rdf::LiteralsOf<'b, I>
	where
		Self: 'b;

	type SubContext<'b>
		= ContextStack<'b, I, D>
	where
		Self: 'b;

	fn iri_interpretation(&self, i: &Iri) -> Option<Self::Resource> {
		self.iri_interpretation(i)
	}

	fn iris_of<'b>(&'b self, r: &'b Self::Resource) -> Self::Iris<'b> {
		self.iris_of(r)
	}

	fn literals_of<'b>(&'b self, r: &'b Self::Resource) -> Self::Literals<'b> {
		self.literals_of(r)
	}

	fn pattern_matching<'b>(
		&'b self,
		pattern: CanonicalQuadPattern<&'b Self::Resource>,
	) -> Self::QuadPatternMatching<'b> {
		self.pattern_matching(pattern)
	}

	fn new_resource(&mut self) -> Self::Resource {
		self.new_resource()
	}

	fn insert_iri<'b>(&mut self, i: impl Into<Cow<'b, Iri>>) -> Self::Resource {
		self.insert_iri(i)
	}

	fn insert_literal<'b>(&mut self, l: impl Into<CowLiteral<'b>>) -> Self::Resource {
		self.insert_literal(l)
	}

	fn insert(&mut self, quad: Quad<Self::Resource>) {
		self.insert(quad)
	}

	fn unify(&mut self, a: &Self::Resource, b: &Self::Resource) {
		self.unify(a, b);
	}

	fn begin(&self) -> Self::SubContext<'_> {
		self.push()
	}
}

pub trait Eval<P, R: Clone> {
	fn eval(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl EvalContext<Resource = R>,
		scope: &Scope<P, R>,
	) -> Result<Value<R>, EvalError<R>>;

	fn eval_inverse(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl EvalContext<Resource = R>,
		scope: &mut Scope<P, R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>>;
}

pub trait EvalTyped<P, R: Clone> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl EvalContext<Resource = R>,
		scope: &Scope<P, R>,
		r#type: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>>;

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl EvalContext<Resource = R>,
		scope: &mut Scope<P, R>,
		r#type: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>>;
}

impl<P: Copy + Ord, R: Ord + Clone> Eval<P, R> for Expr<P, R> {
	fn eval(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl EvalContext<Resource = R>,
		scope: &Scope<P, R>,
	) -> Result<Value<R>, EvalError<R>> {
		self.inner.eval_typed(registry, rdf, scope, self.r#type)
	}

	fn eval_inverse(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl EvalContext<Resource = R>,
		scope: &mut Scope<P, R>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		self.inner
			.eval_inverse_typed(registry, rdf, scope, self.r#type, output)
	}
}

impl<P: Copy + Ord, R: Ord + Clone> EvalTyped<P, R> for InnerExpr<P, R> {
	fn eval_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &impl EvalContext<Resource = R>,
		scope: &Scope<P, R>,
		r#type: TypeRef<P>,
	) -> Result<Value<R>, EvalError<R>> {
		match self {
			Self::Let(bindings, e) => {
				let mut sub_context = rdf.begin();
				let sub_scope = bindings
					.eval(registry, &mut sub_context, scope)?
					.ok_or_else(|| {
						let mut substitution = DebugSubstitution::new();
						EvalError::MissingData(MissingData::Empty(scope.substitute_dataset(
							&sub_context,
							&mut substitution,
							&bindings.dataset,
						)))
					})?;
				e.eval(registry, &sub_context, &sub_scope)
			}
			Self::Var(i) => Ok(scope.get(*i).unwrap().clone()),
			Self::Resource(r) => Ok(Value::Resource(r.clone())),
			Self::Literal(l) => l.eval_typed(registry, rdf, scope, r#type),
			Self::List(list) => list.eval_typed(registry, rdf, scope, r#type),
			Self::Map(map) => map.eval_typed(registry, rdf, scope, r#type),
			Self::Match(m) => m.eval_typed(registry, rdf, scope, r#type),
			Self::Call(call) => call.eval(registry, rdf, scope),
		}
	}

	fn eval_inverse_typed(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl EvalContext<Resource = R>,
		scope: &mut Scope<P, R>,
		r#type: TypeRef<P>,
		output: &Value<R>,
	) -> Result<(), EvalError<R>> {
		match self {
			Self::Let(bindings, e) => bindings.eval_inverse(registry, rdf, scope, |rdf, scope| {
				e.eval_inverse(registry, rdf, scope, output)
			}),
			Self::Var(i) => scope.set(rdf, *i, output.clone()),
			Self::Resource(a) => match output {
				Value::Resource(b) => {
					rdf.unify(a, b);
					// .map_err(|_| EvalError::ResourceUnificationError)
					Ok(())
				}
				_ => Err(EvalError::InvalidType),
			},
			Self::Literal(l) => l.eval_inverse_typed(registry, rdf, scope, r#type, output),
			Self::List(list) => list.eval_inverse_typed(registry, rdf, scope, r#type, output),
			Self::Map(map) => map.eval_inverse_typed(registry, rdf, scope, r#type, output),
			Self::Match(m) => m.eval_inverse_typed(registry, rdf, scope, r#type, output),
			Self::Call(call) => call.eval_inverse(registry, rdf, scope, output),
		}
	}
}
