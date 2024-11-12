use core::fmt;
use rdf_types::{
	dataset::{DatasetMut, PatternMatchingDataset},
	Quad,
};
use std::hash::Hash;

use crate::{
	display::DisplaySignature,
	eval::{EvalError, Scope, ScopeTypes, ScopeValues},
	rdf::{RdfContext, RdfContextMut},
	ty::{BuiltInType, TypeRef},
	Attributes, DatasetPattern, Layout, Registry, Value,
};

mod body;
mod build_in;

pub use body::*;
pub use build_in::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signature<R, T = TypeRef> {
	pub const_args: Vec<T>,
	pub args: Vec<T>,
	pub return_type: T,
	pub dataset: DatasetPattern<R>,
}

impl<R, T> Signature<R, T> {
	pub fn arity(&self) -> u32 {
		self.const_args.len() as u32 + self.args.len() as u32
	}

	pub fn mapped_types<U>(&self, mut f: impl FnMut(&T) -> U) -> Signature<R, U>
	where
		R: Clone,
	{
		Signature {
			const_args: self.const_args.iter().map(&mut f).collect(),
			args: self.args.iter().map(&mut f).collect(),
			return_type: f(&self.return_type),
			dataset: self.dataset.clone(),
		}
	}

	pub fn map_types<U>(self, mut f: impl FnMut(T) -> U) -> Signature<R, U> {
		Signature {
			const_args: self.const_args.into_iter().map(&mut f).collect(),
			args: self.args.into_iter().map(&mut f).collect(),
			return_type: f(self.return_type),
			dataset: self.dataset,
		}
	}
}

impl<R: fmt::Display> DisplaySignature<R> for Signature<R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut core::fmt::Formatter,
	) -> core::fmt::Result {
		write!(f, "(")?;

		for (i, a) in self.args.iter().enumerate() {
			if i > 0 {
				write!(f, " ")?;
			}

			a.fmt_signature(&context.inner(), f)?;
		}

		write!(
			f,
			") -> {}",
			self.return_type.display_signature_with(context.inner())
		)
	}
}

/// Function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Function<R, T = TypeRef> {
	pub attributes: Attributes<R>,
	pub signature: Signature<R, T>,
	pub body: Body<R, T>,
}

impl<R: Ord, T> Function<R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Function<R, U> {
		Function {
			attributes: self.attributes,
			signature: self.signature.map_types(&mut f),
			body: self.body.map_types(f),
		}
	}
}

impl<R> Function<R> {
	pub fn built_in(b: BuiltInFunction, ty: impl Fn(BuiltInType) -> TypeRef) -> Self {
		Self {
			attributes: Attributes::new(),
			signature: b.signature(ty),
			body: Body::BuiltIn(b),
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
		let arg_count = args.len() as u32;
		if args.len() as u32 != self.signature.arity() {
			return Err(EvalError::ArgumentCount {
				expected: self.signature.arity(),
				found: arg_count,
			});
		}

		// TODO check signature dataset

		let scope = Scope::new(
			None,
			ScopeTypes::TwoSlices(&self.signature.const_args, &self.signature.args),
			ScopeValues::ConstBorrowed(args),
		);

		self.body.eval(registry, rdf, dataset, &scope)
	}

	pub fn call_inverse(
		&self,
		registry: &Registry<R>,
		rdf: &mut impl RdfContextMut<R>,
		dataset: &mut impl DatasetMut<Resource = R>,
		const_args: &[Value<R>],
		output: &Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>> {
		if const_args.len() != self.signature.const_args.len() {
			todo!()
		}

		Scope::root(None)
			.begin(
				None,
				ScopeTypes::Slice(&self.signature.const_args),
				ScopeValues::ConstBorrowed(const_args),
			)
			.begin_mut(
				rdf,
				dataset,
				None,
				ScopeTypes::Slice(&self.signature.args),
				|rdf, dataset, scope| {
					scope.extend_dataset(self.signature.dataset.iter().map(Quad::into_cloned));
					self.body
						.eval_inverse(registry, rdf, dataset, scope, output)
				},
			)
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
	pub index: u32,
}

impl FunctionRef {
	pub fn new(namespace: u32, index: u32) -> Self {
		Self { namespace, index }
	}
}

impl<R: fmt::Display> DisplaySignature<R> for FunctionRef {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut core::fmt::Formatter,
	) -> core::fmt::Result {
		let function = context.registry.get_function(*self).unwrap();
		function.signature.fmt_signature(context, f)
	}
}
