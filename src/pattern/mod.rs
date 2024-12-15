mod rdf;
use std::hash::Hash;

pub use rdf::*;

pub mod substitution;
use serde::{Deserialize, Serialize};
use serde_seeded::{unseeded, DeserializeSeeded, SerializeSeeded};
pub use substitution::Substitution;

use crate::{
	expr::Expr,
	function::FunctionRef,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex},
	value::TypedValueMap,
	TypeRef,
};

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct Pattern<P, R, T = TypeRef<P>> {
	#[seeded(with(unseeded))]
	pub r#type: T,
	pub desc: PatternDesc<P, R, T>,
}

impl<P, R, T> Pattern<P, R, T> {
	pub fn new(r#type: T, desc: PatternDesc<P, R, T>) -> Self {
		Self { r#type, desc }
	}

	pub fn visit_variables(&self, mut f: impl FnMut(u32)) {
		self.visit_variables_with(&mut f);
	}

	fn visit_variables_with(&self, f: &mut impl FnMut(u32)) {
		self.desc.visit_variables_with(f);
	}

	pub fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> Pattern<P, R, U>
	where
		R: Ord,
	{
		Pattern {
			r#type: f(self.r#type),
			desc: self.desc.map_types_ref_mut(f),
		}
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub enum PatternDesc<P, R, T = TypeRef<P>> {
	Var(u32),
	List(Vec<Pattern<P, R, T>>),
	Map(MapPattern<P, R, T>),
	Call(CallPattern<P, R, T>),
}

impl<P, R, T> PatternDesc<P, R, T> {
	fn visit_variables_with(&self, f: &mut impl FnMut(u32)) {
		match self {
			Self::Var(x) => f(*x),
			Self::List(items) => {
				for item in items {
					item.visit_variables_with(f);
				}
			}
			Self::Map(entries) => {
				for (_, value) in entries {
					value.visit_variables_with(f);
				}
			}
			Self::Call(call) => call.visit_variables_with(f),
		}
	}

	fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> PatternDesc<P, R, U>
	where
		R: Ord,
	{
		match self {
			Self::Var(x) => PatternDesc::Var(x),
			Self::List(items) => PatternDesc::List(
				items
					.into_iter()
					.map(|item| item.map_types_ref_mut(f))
					.collect(),
			),
			Self::Map(entries) => PatternDesc::Map(
				entries
					.into_iter()
					.map(|(key, value)| (key.map_types_ref_mut(f), value.map_types_ref_mut(f)))
					.collect(),
			),
			Self::Call(call) => PatternDesc::Call(call.map_types_ref_mut(f)),
		}
	}
}

pub type MapPattern<P, R, T = TypeRef<P>> = TypedValueMap<P, R, Pattern<P, R, T>, T>;

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct CallPattern<P, R, T = TypeRef<P>> {
	pub function: FunctionRef<P>,
	pub const_args: Vec<Expr<P, R, T>>,
	pub args: Vec<Pattern<P, R, T>>,
}

impl<P, R, T> CallPattern<P, R, T> {
	fn visit_variables_with(&self, f: &mut impl FnMut(u32)) {
		for a in &self.args {
			a.visit_variables_with(f);
		}
	}

	fn map_types_ref_mut<U: Ord>(self, f: &mut impl FnMut(T) -> U) -> CallPattern<P, R, U>
	where
		R: Ord,
	{
		CallPattern {
			function: self.function,
			const_args: self
				.const_args
				.into_iter()
				.map(|e| e.map_types_ref_mut(f))
				.collect(),
			args: self
				.args
				.into_iter()
				.map(|a| a.map_types_ref_mut(f))
				.collect(),
		}
	}
}
