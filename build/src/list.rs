use crate::{error, Context, Descriptions, Error};
use locspan::Meta;
use treeldr::{vocab::Object, Id, MetaOption};

#[derive(Clone)]
pub struct Definition<M> {
	id: Id,
	first: MetaOption<Object<M>, M>,
	rest: MetaOption<Id, M>,
}

impl<M> Definition<M> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			first: MetaOption::default(),
			rest: MetaOption::default(),
		}
	}

	pub fn set_first(&mut self, object: Object<M>, cause: M) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.first.try_set_stripped(
			object,
			cause,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
				Meta(
					error::ListMismatchItem {
						id: self.id,
						expected: expected.clone(),
						found,
						because: expected_meta.clone(),
					}
					.into(),
					found_meta,
				)
			},
		)
	}

	pub fn set_rest(&mut self, list: Id, cause: M) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.rest.try_set(
			list,
			cause,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::ListMismatchRest {
						id: self.id,
						expected,
						found,
						because: expected_meta,
					}
					.into(),
					found_meta,
				)
			},
		)
	}
}

pub enum ListRef<'l, M> {
	Nil,
	Cons(&'l Meta<Definition<M>, M>),
}

impl<'l, M> ListRef<'l, M> {
	pub fn iter<C: RequireList<M>>(&self, nodes: &'l C) -> Iter<'l, M, C> {
		match self {
			Self::Nil => Iter::Nil,
			Self::Cons(l) => Iter::Cons(nodes, l),
		}
	}
}

pub trait RequireList<M> {
	fn require_list(&self, id: Id, cause: &M) -> Result<ListRef<M>, Error<M>>
	where
		M: Clone;
}

impl<M, D: Descriptions<M>> RequireList<M> for Context<M, D> {
	fn require_list(&self, id: Id, cause: &M) -> Result<ListRef<M>, Error<M>>
	where
		M: Clone,
	{
		self.require_list(id, cause)
	}
}

impl<M: Clone> RequireList<M> for super::context::allocated::Nodes<M> {
	fn require_list(&self, id: Id, cause: &M) -> Result<ListRef<M>, Error<M>> {
		self.require_list(id, cause)
	}
}

pub enum Iter<'l, M, C> {
	Nil,
	Cons(&'l C, &'l Meta<Definition<M>, M>),
}

impl<'l, M: Clone, C: RequireList<M>> Iterator for Iter<'l, M, C> {
	type Item = Result<&'l Meta<Object<M>, M>, Error<M>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Nil => None,
			Self::Cons(nodes, d) => {
				let item = d.first.value_or_else(|| {
					Meta(error::ListMissingItem(d.id).into(), d.metadata().clone())
				});

				let rest_id = d.rest.value_or_else(|| {
					Meta(error::ListMissingRest(d.id).into(), d.metadata().clone())
				});

				match rest_id {
					Ok(Meta(rest_id, meta)) => {
						match nodes.require_list(*rest_id, meta) {
							Ok(ListRef::Cons(rest)) => *self = Self::Cons(*nodes, rest),
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

pub enum ListMut<'l, M> {
	Nil,
	Cons(&'l mut Definition<M>),
}
