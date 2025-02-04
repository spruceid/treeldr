use core::fmt;
use rdf_types::Quad;
use serde::{Deserialize, Serialize};
use serde_seeded::{unseeded, DeserializeSeeded, SerializeSeeded};
use std::{hash::Hash, marker::PhantomData};

use crate::{
	display::DisplaySignature,
	eval::{EvalError, Scope, ScopeTypes, ScopeValues},
	pattern::serde_dataset_pattern,
	rdf::{
		EvalDataset, RdfContextMut, ResourceBackwardIndex, ResourceForwardIndex, TryUnifyInterpretation, VisitResources
	},
	ty::{BuiltInTypeMap, TypeRef},
	Attributes, DatasetPattern, Registry, Value,
};

mod body;
mod build_in;

pub use body::*;
pub use build_in::*;

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct Signature<P, R, T = TypeRef<P>> {
	#[seeded(with(unseeded))]
	pub const_args: Vec<T>,

	#[seeded(with(unseeded))]
	pub args: Vec<T>,

	#[seeded(with(unseeded))]
	pub return_type: T,

	#[seeded(with(serde_dataset_pattern))]
	pub dataset: DatasetPattern<R>,

	#[seeded(skip)]
	p: PhantomData<P>,
}

impl<P, R, T> Signature<P, R, T> {
	pub fn new(
		const_args: Vec<T>,
		args: Vec<T>,
		return_type: T,
		dataset: DatasetPattern<R>,
	) -> Self {
		Self {
			const_args,
			args,
			return_type,
			dataset,
			p: PhantomData,
		}
	}

	pub fn arity(&self) -> u32 {
		self.const_args.len() as u32 + self.args.len() as u32
	}

	pub fn mapped_types<U>(&self, mut f: impl FnMut(&T) -> U) -> Signature<P, R, U>
	where
		R: Clone,
	{
		Signature {
			const_args: self.const_args.iter().map(&mut f).collect(),
			args: self.args.iter().map(&mut f).collect(),
			return_type: f(&self.return_type),
			dataset: self.dataset.clone(),
			p: PhantomData,
		}
	}

	pub fn map_types<U>(self, mut f: impl FnMut(T) -> U) -> Signature<P, R, U> {
		Signature {
			const_args: self.const_args.into_iter().map(&mut f).collect(),
			args: self.args.into_iter().map(&mut f).collect(),
			return_type: f(self.return_type),
			dataset: self.dataset,
			p: PhantomData,
		}
	}
}

impl<P, R, T> VisitResources<R> for Signature<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.dataset.visit_resources_ref_mut(f)
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for Signature<P, R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
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
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>), bounds(T: Ord))
)]
pub struct Function<P, R, T = TypeRef<P>> {
	pub attributes: Attributes<R>,
	pub signature: Signature<P, R, T>,
	pub body: Body<P, R, T>,
}

impl<P, R: Ord, T> Function<P, R, T> {
	pub fn map_types<U: Ord>(self, mut f: impl FnMut(T) -> U) -> Function<P, R, U> {
		Function {
			attributes: self.attributes,
			signature: self.signature.map_types(&mut f),
			body: self.body.map_types(f),
		}
	}
}

impl<P, R> Function<P, R> {
	pub fn built_in(b: BuiltInFunction, ty: &BuiltInTypeMap<TypeRef<P>>) -> Self where P: Copy {
		Self {
			attributes: Attributes::new(),
			signature: b.signature(ty),
			body: Body::BuiltIn(b),
		}
	}
}

impl<P: Copy + Ord, R: Clone + Ord> Function<P, R> {
	pub fn call(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		args: &[Value<R>],
	) -> Result<Value<R>, EvalError<R>> {
		let mut result = self.call_without_unification(registry, rdf, dataset, args)?;
		dataset.apply_unification(rdf.interpretation());
		result.map_resources(|r| rdf.interpretation().substitute_resource(r));
		Ok(result)
	}

	pub fn call_without_unification(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
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

	/// Call the inverse function.
	///
	/// Returns the inputs associated to the given output and const arguments.
	/// The returned input list does *not* include the const arguments again.
	pub fn call_inverse(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
		const_args: &[Value<R>],
		output: &Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>> {
		let mut result =
			self.call_inverse_without_unification(registry, rdf, dataset, const_args, output)?;
		
		dataset.apply_unification(rdf.interpretation());
		for value in &mut result {
			value.map_resources(|r| rdf.interpretation().substitute_resource(r));
		}

		Ok(result)
	}

	/// Call the inverse function.
	///
	/// Returns the inputs associated to the given output and const arguments.
	/// The returned input list does *not* include the const arguments again.
	pub fn call_inverse_without_unification(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		rdf: &mut impl RdfContextMut<Resource = R>,
		dataset: &mut impl EvalDataset<R>,
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

impl<P, R, T> VisitResources<R> for Function<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.attributes.visit_resources_ref_mut(f);
		self.signature.visit_resources_ref_mut(f);
		self.body.visit_resources_ref_mut(f)
	}
}

/// Function argument.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arg<T> {
	pub r#type: T,
	pub is_const: bool,
}

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(params(Q), seed(Q), override_bounds(P: Serialize)),
	de(params(Q), seed(Q), override_bounds(P: Deserialize<'de>))
)]
pub struct FunctionRef<P> {
	#[seeded(with(unseeded))]
	pub package: P,
	pub index: u32,
}

impl<P> FunctionRef<P> {
	pub fn new(package: P, index: u32) -> Self {
		Self { package, index }
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for FunctionRef<P> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		let function = context.registry.get_function(*self).unwrap();
		function.signature.fmt_signature(context, f)
	}
}
