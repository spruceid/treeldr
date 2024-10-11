use std::{borrow::Cow, cell::OnceCell, hash::Hash, ops::Deref, sync::Arc};
use educe::Educe;
use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

use crate::{
	eval::{EvalError, RdfContext, RdfContextMut, Scope, ScopeTypes},
	ty::TypeRef,
	Domain, Layout, Value,
};

mod body;
mod build_in;

pub use body::*;
pub use build_in::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signature<T> {
	pub args: Vec<T>,
	pub return_type: T
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
#[educe(PartialOrd(bound = "R: 'static + PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord"))]
pub struct Function<R> {
	pub signature: Signature<TypeRef<R>>,
	pub body: Body<R>,
}

impl<R> Function<R> {
	pub fn as_layout(&self) -> Option<&Layout<R>> {
		for a in &self.signature.args {
			if a.is_resource() {
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
		self.body
			.eval(rdf, dataset, &scope)
	}

	pub fn call_inverse(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		output: &Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>> {
		let input_types: Vec<_> = self.signature.args.iter().map(|a| a.clone()).collect();
		self.body.eval_inverse(
			rdf,
			dataset,
			&input_types,
			output,
		)
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

#[derive(Debug, Educe)]
#[educe(Clone)]
pub struct FunctionRef<R>(Arc<OnceCell<Function<R>>>);

impl<R> FunctionRef<R> {
	pub fn new(value: Function<R>) -> Self {
		todo!()
	}

	pub fn new_undefined() -> Self {
		todo!()
	}

	pub fn define(&self, value: Function<R>) {
		todo!()
	}
}

impl<R> Deref for FunctionRef<R> {
	type Target = Function<R>;
	
	fn deref(&self) -> &Self::Target {
		self.0.get().expect("undefined function")
	}
}

impl<R: PartialEq> PartialEq for FunctionRef<R> {
	fn eq(&self, other: &Self) -> bool {
		Function::<R>::eq(self, other)
	}
}

impl<R: Eq> Eq for FunctionRef<R> {}

impl<R: 'static + PartialOrd> PartialOrd for FunctionRef<R> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		<Function<R>>::partial_cmp(self, other)
	}
}

impl<R: 'static + Ord> Ord for FunctionRef<R> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		<Function<R>>::cmp(self, other)
	}
}

impl<R: Hash> Hash for FunctionRef<R> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		<Function<R>>::hash(self, state);
	}
}