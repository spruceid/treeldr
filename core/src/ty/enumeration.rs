use crate::{Causes, Value};
use std::collections::BTreeMap;

/// Enumeration type.
pub struct Enumeration<F> {
	values: BTreeMap<Value, Causes<F>>
}

impl<F> Enumeration<F> {
	pub fn new(
		values: BTreeMap<Value, Causes<F>>
	) -> Self {
		Self {
			values
		}
	}

	pub fn values(&self) -> &BTreeMap<Value, Causes<F>> {
		&self.values
	}

	pub fn is_datatype(&self) -> bool {
		self.values.iter().all(|(v, _)| v.is_literal())
	}
}
