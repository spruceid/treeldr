use std::collections::BTreeMap;

use treeldr::{
	layout::{ListLayout, LiteralLayout, ProductLayout, SumLayout},
	Ref,
};

/// Untyped value.
pub enum Value {
	Literal(Vec<u8>),
	Record(BTreeMap<String, Self>),
	List(Vec<Self>),
}

/// Typed value.
pub enum TypedValue<R> {
	Literal(Vec<u8>, Ref<R, LiteralLayout<R>>),
	Variant(Box<Self>, Ref<R, SumLayout<R>>),
	Record(BTreeMap<String, Self>, Ref<R, ProductLayout<R>>),
	List(Vec<Self>, Ref<R, ListLayout<R>>),
}
