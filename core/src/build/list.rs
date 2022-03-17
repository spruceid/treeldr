use super::{error, Error};
use crate::{vocab::Object, Caused, Id, MaybeSet, WithCauses};
use locspan::Location;

pub struct Definition<F> {
	id: Id,
	first: MaybeSet<Object<F>, F>,
	rest: MaybeSet<Id, F>,
}

impl<F> Definition<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			first: MaybeSet::default(),
			rest: MaybeSet::default(),
		}
	}

	pub fn set_first(
		&mut self,
		object: Object<F>,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.first
			.try_set_stripped(object, cause, |expected, because, found| {
				error::ListMismatchItem {
					id: self.id,
					expected: expected.clone(),
					found,
					because: because.cloned(),
				}
				.into()
			})
	}

	pub fn set_rest(&mut self, list: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.rest.try_set(list, cause, |expected, because, found| {
			error::ListMismatchRest {
				id: self.id,
				expected: *expected,
				found,
				because: because.cloned(),
			}
			.into()
		})
	}
}

pub enum ListRef<'l, F> {
	Nil,
	Cons(&'l WithCauses<Definition<F>, F>),
}

impl<'l, F> ListRef<'l, F> {
	pub fn iter(&self, nodes: &'l super::context::AllocatedNodes<F>) -> Iter<'l, F> {
		match self {
			Self::Nil => Iter::Nil,
			Self::Cons(l) => Iter::Cons(nodes, l),
		}
	}
}

pub enum Iter<'l, F> {
	Nil,
	Cons(
		&'l super::context::AllocatedNodes<F>,
		&'l WithCauses<Definition<F>, F>,
	),
}

impl<'l, F: Clone> Iterator for Iter<'l, F> {
	type Item = Result<&'l WithCauses<Object<F>, F>, Error<F>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Nil => None,
			Self::Cons(nodes, d) => {
				let item = d.first.value_or_else(|| {
					Caused::new(
						error::ListMissingItem(d.id).into(),
						d.causes().preferred().cloned(),
					)
				});

				let rest_id = d.rest.value_or_else(|| {
					Caused::new(
						error::ListMissingRest(d.id).into(),
						d.causes().preferred().cloned(),
					)
				});

				match rest_id {
					Ok(rest_id) => {
						match nodes
							.require_list(*rest_id.inner(), rest_id.causes().preferred().cloned())
						{
							Ok(ListRef::Cons(rest)) => *self = Self::Cons(nodes, rest),
							Ok(ListRef::Nil) => *self = Self::Nil,
							Err(e) => return Some(Err(e)),
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
	Cons(&'l mut Definition<F>),
}
