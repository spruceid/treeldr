use rdf_types::dataset::{DatasetMut, PatternMatchingDataset};

use crate::{
	eval::{EvalError, Scope}, ty::BuiltInType, TypeRef, Value
};

use super::Signature;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltInFunction {
	StringOf
}

impl BuiltInFunction {
	pub fn signature(
		&self,
		ty: impl Fn(BuiltInType) -> TypeRef
	) -> Signature {
		match self {
			Self::StringOf => {
				Signature {
					args: vec![ty(BuiltInType::Resource)],
					return_type: ty(BuiltInType::TextString)
				}
			}
		}
	}

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
