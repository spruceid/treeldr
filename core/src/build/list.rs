use crate::{Id, MaybeSet, Caused, WithCauses, vocab::Object};
use locspan::Location;
use super::Error;

pub struct Definition<F> {
	first: MaybeSet<Object<F>, F>,
	rest: MaybeSet<Id, F>
}

impl<F> Definition<F> {
	pub fn new() -> Self {
		Self {
			first: MaybeSet::default(),
			rest: MaybeSet::default()
		}
	}

	pub fn set_first(&mut self, object: Object<F>, cause: Option<Location<F>>) -> Result<(), Caused<Error<F>, F>> where F: Clone + Ord {
		self.first.try_set_stripped(object, cause, |expected, because, found| Error::ListItemMismatch {
			expected: expected.clone(),
			found,
			because: because.cloned()
		})
	}

	pub fn set_rest(&mut self, list: Id, cause: Option<Location<F>>) -> Result<(), Caused<Error<F>, F>> where F: Clone + Ord {
		self.rest.try_set(list, cause, |expected, because, found| Error::ListRestMismatch {
			expected: expected.clone(),
			found,
			because: because.cloned()
		})
	}
}

pub enum ListRef<'l, F> {
	Nil,
	Cons(&'l WithCauses<Definition<F>, F>)
}

impl<'l, F> ListRef<'l, F> {
	pub fn iter(&self, nodes: &'l super::context::AllocatedNodes<F>) -> Iter<'l, F> {
		match self {
			Self::Nil => Iter::Nil,
			Self::Cons(l) => Iter::Cons(nodes, l)
		}
	}
}

pub enum Iter<'l, F> {
	Nil,
	Cons(&'l super::context::AllocatedNodes<F>, &'l WithCauses<Definition<F>, F>)
}

impl<'l, F: Clone> Iterator for Iter<'l, F> {
	type Item = Result<&'l WithCauses<Object<F>, F>, Caused<Error<F>, F>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Nil => None,
			Self::Cons(nodes, d) => {
				let item = d.first.value_or_else(|| Caused::new(
					Error::Unimplemented(crate::Feature::Error("missing list item")),
					d.causes().preferred().cloned()
				));

				let rest_id = d.rest.value_or_else(|| Caused::new(
					Error::Unimplemented(crate::Feature::Error("missing list rest")),
					d.causes().preferred().cloned()
				));

				match rest_id {
					Ok(rest_id) => {
						match nodes.require_list(*rest_id.inner(), rest_id.into_causes().into_preferred()) {
							Ok(ListRef::Cons(rest)) => *self = Self::Cons(nodes, rest),
							Ok(ListRef::Nil) => *self = Self::Nil,
							Err(e) => return Some(Err(e))
						}

						Some(item)
					}
					Err(e) => Some(Err(e)),

				}
			}
		}
	}
}

pub enum ListMut<'l, F> {
	Nil,
	Cons(&'l mut Definition<F>)
}