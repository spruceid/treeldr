use crate::{Causes, Ref};
use std::collections::HashMap;

/// Intersection type.
pub struct Intersection<F> {
	/// Types in the intersection.
	types: HashMap<Ref<super::Definition<F>>, Causes<F>>,
}

impl<F> Intersection<F> {
	pub fn new(types: HashMap<Ref<super::Definition<F>>, Causes<F>>) -> Self {
		Self { types }
	}
}