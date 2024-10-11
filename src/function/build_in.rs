use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

use crate::{eval::{EvalError, Scope}, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltInFunction {
	// ...
}

impl BuiltInFunction {
	pub fn eval<R: Ord + Clone>(
		&self,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		scope: &Scope<R>,
	) -> Result<Value<R>, EvalError<R>> {
		todo!()
	}

	pub fn eval_inverse<R>(
		&self,
		dataset: &mut impl DatasetMut<Resource = R>,
		output: &Value<R>,
	) -> Result<Vec<Value<R>>, EvalError<R>> {
		todo!()
	}
}
