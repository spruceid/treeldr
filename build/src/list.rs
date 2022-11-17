use crate::{Context, Error, Single};
use locspan::{Meta, Stripped};
use treeldr::{vocab::Object, Id};

pub use treeldr::list::Property;

#[derive(Clone)]
pub struct Definition<M> {
	first: Single<Stripped<Object<M>>, M>,
	rest: Single<Id, M>,
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self {
			first: Single::default(),
			rest: Single::default(),
		}
	}

	pub fn first(&self) -> &Single<Stripped<Object<M>>, M> {
		&self.first
	}

	pub fn first_mut(&mut self) -> &mut Single<Stripped<Object<M>>, M> {
		&mut self.first
	}

	pub fn rest(&self) -> &Single<Id, M> {
		&self.rest
	}

	pub fn rest_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.rest
	}
}

pub enum ListRef<'l, M> {
	Nil,
	Cons(Id, &'l Definition<M>, &'l M),
}

impl<'l, M> ListRef<'l, M> {
	pub fn iter(&self, context: &'l Context<M>) -> Iter<'l, M> {
		match self {
			Self::Nil => Iter::Nil,
			Self::Cons(id, l, meta) => Iter::Cons(context, *id, *l, *meta),
		}
	}

	pub fn lenient_iter(&self, context: &'l Context<M>) -> LenientIter<'l, M> {
		match self {
			Self::Nil => LenientIter::Nil,
			Self::Cons(id, l, meta) => LenientIter::Cons(context, *l),
		}
	}
}

pub enum Iter<'l, M> {
	Nil,
	Cons(&'l Context<M>, Id, &'l Definition<M>, &'l M),
}

impl<'l, M: Clone> Iterator for Iter<'l, M> {
	type Item = Result<Meta<&'l Object<M>, &'l M>, Error<M>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Nil => None,
			Self::Cons(nodes, id, d, meta) => {
				match d.first.as_required_at_node_binding(*id, Property::First, meta) {
					Ok(item) => {
						match d.rest.as_required_at_node_binding(*id, Property::Rest, meta) {
							Ok(Meta(rest_id, _)) => {
								match nodes.require_list(*rest_id) {
									Ok(ListRef::Cons(rest_id, rest, rest_meta)) => *self = Self::Cons(*nodes, rest_id, rest, rest_meta),
									Ok(ListRef::Nil) => *self = Self::Nil,
									Err(e) => return Some(Err(e.at_node_property(*id, Property::Rest, meta.clone())))
								}

								Some(Ok(item.map(Stripped::as_ref)))
							}
							Err(e) => Some(Err(e))
						}
					}
					Err(e) => Some(Err(e))
				}
			}
		}
	}
}

pub enum LenientIter<'l, M> {
	Nil,
	Cons(&'l Context<M>, &'l Definition<M>),
}

impl<'l, M> Iterator for LenientIter<'l, M> {
	type Item = Meta<&'l Object<M>, &'l M>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self {
				Self::Nil => break None,
				Self::Cons(nodes, d) => {
					let item = d.first.first();

					match d.rest.first().and_then(|rest| nodes.get_list(**rest)) {
						Some(ListRef::Cons(_, rest, _)) => *self = Self::Cons(*nodes, rest),
						_ => *self = Self::Nil
					}

					if let Some(item) = item {
						break Some(item.map(Stripped::as_ref))
					}
				}
			}
		}
	}
}

pub enum ListMut<'l, M> {
	Nil,
	Cons(&'l mut Definition<M>),
}

pub enum BindingRef<'a, M> {
	First(&'a Object<M>),
	Rest(Id),
}

impl<'a, M> BindingRef<'a, M> {
	pub fn property(&self) -> Property {
		match self {
			Self::First(_) => Property::First,
			Self::Rest(_) => Property::Rest,
		}
	}
}

/// Iterator over the bindings of a given list.
pub struct Bindings<'a, M> {
	first: Option<&'a Object<M>>,
	rest: Option<Id>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = BindingRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.first
			.take()
			.map(BindingRef::First)
			.or_else(|| self.rest.take().map(BindingRef::Rest))
	}
}