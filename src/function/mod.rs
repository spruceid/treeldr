use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};
use std::{borrow::Cow, hash::Hash};

use crate::{
	eval::{EvalError, Scope, ScopeTypes}, rdf::{RdfContext, RdfContextMut}, ty::{BuiltInType, TypeRef}, Layout, Registry, Value
};

mod body;
mod build_in;

pub use body::*;
pub use build_in::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signature<T = TypeRef> {
	pub args: Vec<T>,
	pub return_type: T,
}

impl<T> Signature<T> {
	pub fn arity(&self) -> u32 {
		self.args.len() as u32
	}

	pub fn mapped_types<U>(&self, mut f: impl FnMut(&T) -> U) -> Signature<U> {
		Signature {
			args: self.args.iter().map(&mut f).collect(),
			return_type: f(&self.return_type)
		}
	}

	pub fn map_types<U>(self, mut f: impl FnMut(T) -> U) -> Signature<U> {
		Signature {
			args: self.args.into_iter().map(&mut f).collect(),
			return_type: f(self.return_type)
		}
	}
}

/// Function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Function<R, T = TypeRef> {
	pub signature: Signature<T>,
	pub body: Body<R, T>,
}

impl<R> Function<R> {
	pub fn built_in(b: BuiltInFunction, ty: impl Fn(BuiltInType) -> TypeRef) -> Self {
		Self {
			signature: b.signature(ty),
			body: Body::BuiltIn(b)
		}
	}

	pub fn as_layout(&self, registry: &Registry<R>) -> Option<&Layout<R>> {
		for &a in &self.signature.args {
			if registry.get_type(a)?.is_resource() {
				return None;
			}
		}

		Some(self.as_layout_unchecked())
	}

	fn as_layout_unchecked(&self) -> &Layout<R> {
		unsafe { std::mem::transmute::<&Function<R>, &Layout<R>>(self) }
	}
}

impl<R: Clone + Ord> Function<R> {
	pub fn call(
		&self,
		registry: &Registry<R>,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		args: &[Value<R>],
	) -> Result<Value<R>, EvalError<R>> {
		if args.len() != self.signature.args.len() {
			return Err(EvalError::ArgumentCount {
				expected: self.signature.args.len(),
				found: args.len()
			})
		}

		let scope = Scope::new(
			None,
			None,
			ScopeTypes::Slice(&self.signature.args),
			Cow::Borrowed(args),
		);
		self.body.eval(registry, rdf, dataset, &scope)
	}

	pub fn call_inverse(
		&self,
		registry: &Registry<R>,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		output: &Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>> {
		let input_types: Vec<_> = self.signature.args.iter().map(|a| a.clone()).collect();
		self.body
			.eval_inverse(registry, rdf, dataset, &input_types, output)
	}
}

/// Function argument.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arg<T> {
	pub type_: T,
	pub is_const: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionRef {
	pub namespace: u32,
	pub index: u32
}

impl FunctionRef {
	pub fn new(namespace: u32, index: u32) -> Self {
		Self {
			namespace,
			index
		}
	}
}