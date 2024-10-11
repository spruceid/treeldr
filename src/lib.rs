pub mod eval;
pub mod expr;
pub mod matching;
pub mod pattern;
pub mod syntax;
pub mod ty;
pub mod utils;
pub mod value;
pub mod function;
pub mod layout;

use std::{collections::BTreeMap, sync::Arc};

use educe::Educe;
pub use matching::Matching;
pub use pattern::{DatasetPattern, Graph, TermPattern};
pub use ty::{Type, TypeRef};
pub use value::{Literal, Value};
pub use function::Function;
pub use layout::Layout;

pub trait Domain {
	type Resource;
}

#[derive(Debug, Educe)]
#[educe(Default)]
pub struct Module<R> {
	pub functions: BTreeMap<String, Arc<Function<R>>>
}

impl<R> Module<R> {
	// ...
}