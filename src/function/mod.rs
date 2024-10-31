use educe::Educe;
use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};
use std::{borrow::Cow, cell::OnceCell, hash::Hash, ops::Deref, sync::Arc};

use crate::{
	eval::{EvalError, RdfContext, RdfContextMut, Scope, ScopeTypes},
	ty::TypeRef,
	Domain, Layout, Registry, Value,
};

mod body;
mod build_in;

pub use body::*;
pub use build_in::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signature<T> {
	pub args: Vec<T>,
	pub return_type: T,
}

impl<R> Signature<R> {
	pub fn arity(&self) -> u32 {
		self.args.len() as u32
	}
}

impl<R> Domain for Signature<R> {
	type Resource = R;
}

/// Function.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub struct Function<R, T = TypeRef> {
	pub signature: Signature<T>,
	pub body: Body<R, T>,
}

impl<R> Function<R> {
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

impl<R> Domain for Function<R> {
	type Resource = R;
}

/// Function argument.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arg<T> {
	pub type_: T,
	pub is_const: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionRef(usize);

impl From<FunctionRef> for usize {
	fn from(value: FunctionRef) -> Self {
		value.0
	}
}

impl From<usize> for FunctionRef {
	fn from(value: usize) -> Self {
		Self(value)
	}
}
