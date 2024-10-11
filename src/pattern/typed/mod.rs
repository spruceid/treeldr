use std::hash::Hash;
use crate::{value::{TypedValue, TypedValueDesc, TypedValueInnerDesc}, Literal, TypeRef, Value};

mod list;
pub use list::*;

mod map;
pub use map::*;

/// Typed pattern.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypedPattern<R> {
	pub type_: TypeRef<R>,
	pub desc: TypedPatternDesc<R>
}

impl<R> TypedPattern<R> {
	pub fn matches(&self, value: &TypedValue<R>) -> bool
	where
		R: Ord
	{
		if value.type_.is_subtype_of(&self.type_) {
			match (&self.desc, &value.desc) {
				(TypedPatternDesc::Bind, _) => true,
				(TypedPatternDesc::Variant(a, pattern), TypedValueDesc::Variant(b, value)) if a == b => {
					pattern.matches(value)
				}
				(TypedPatternDesc::Constant(a), TypedValueDesc::Constant(b)) => {
					match (a, b) {
						(TypedPatternInnerDesc::Resource(a), TypedValueInnerDesc::Resource(b)) => {
							a == b
						}
						(TypedPatternInnerDesc::Literal(a), TypedValueInnerDesc::Literal(b)) => {
							a == b
						}
						(TypedPatternInnerDesc::List(a), TypedValueInnerDesc::List(b)) => {
							a.matches(b)
						}
						(TypedPatternInnerDesc::Map(a), TypedValueInnerDesc::Map(b)) => {
							a.matches(b)
						}
						_ => false
					}
				}
				_ => false
			}
		} else {
			false
		}
	}

	pub fn is_sub_pattern_of(&self, other: &TypedPattern<R>) -> bool {
		todo!()
	}

	pub fn instantiate(
		&self,
		value: Value<R>
	) -> Result<TypedValue<R>, Value<R>> where R: Clone + Ord + Hash {
		match &self.desc {
			TypedPatternDesc::Bind => {
				value.into_typed(&self.type_)
			}
			TypedPatternDesc::Variant(name, pattern) => {
				let typed_value = pattern.instantiate(value)?;
				Ok(TypedValue {
					type_: self.type_.clone(),
					desc: TypedValueDesc::Variant(name.clone(), Box::new(typed_value))
				})
			}
			TypedPatternDesc::Constant(desc) => {
				match (desc, value) {
					(TypedPatternInnerDesc::Resource(a), Value::Resource(b)) if *a == b => {
						Ok(TypedValue {
							type_: self.type_.clone(),
							desc: TypedValueInnerDesc::Resource(b).into()
						})
					},
					(TypedPatternInnerDesc::Literal(a), Value::Literal(b)) if *a == b => {
						Ok(TypedValue {
							type_: self.type_.clone(),
							desc: TypedValueInnerDesc::Literal(b).into()
						})
					}
					(TypedPatternInnerDesc::List(pattern), Value::List(items)) => {
						let result = pattern
							.instantiate(&self.type_, items)
							.map_err(Value::List)?;
		
						Ok(TypedValue {
							type_: self.type_.clone(),
							desc: TypedValueInnerDesc::List(result).into()
						})
					}
					(TypedPatternInnerDesc::Map(pattern), Value::Map(map)) => {
						let result = pattern
							.instantiate(&self.type_, map)
							.map_err(Value::Map)?;
		
						Ok(TypedValue {
							type_: self.type_.clone(),
							desc: TypedValueInnerDesc::Map(result).into()
						})
					}
					(_, value) => Err(value)
				}
			}
		}
	}
}

/// Untyped tree value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedPatternDesc<R> {
	Bind,
	Variant(String, Box<TypedPattern<R>>),
	Constant(TypedPatternInnerDesc<R>)
}

/// Untyped tree value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedPatternInnerDesc<R> {
	Resource(R),
	Literal(Literal),
	Map(TypedMapPattern<R>),
	List(TypedListPattern<R>)
}