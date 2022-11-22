use crate::{Context, Error, Single, error, single, resource::BindingValueRef, context::MapIds};
use derivative::Derivative;
use locspan::{Meta, Stripped};
use treeldr::{vocab::Object, Id, metadata::Merge};

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

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings { first: self.first.iter(), rest: self.rest.iter() }
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id) -> Id) {
		self.first.map_ids(&f);
		self.rest.map_ids(f)
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub enum ListRef<'l, M> {
	Nil,
	Cons(Id, &'l Definition<M>, &'l M),
}

impl<'l, M> ListRef<'l, M> {
	pub fn try_fold<T, F>(&self, context: &'l Context<M>, first: T, f: F) -> TryFold<'l, T, M, F> where F: Fn(T, &Single<Stripped<Object<M>>, M>) -> Result<Vec<T>, Error<M>> {
		TryFold { context, stack: vec![Ok((*self, first))], f }
	}

	pub fn iter(&self, context: &'l Context<M>) -> Iter<'l, M> {
		match self {
			Self::Nil => Iter::Nil,
			Self::Cons(id, l, meta) => Iter::Cons(context, *id, *l, *meta),
		}
	}

	pub fn lenient_iter(&self, context: &'l Context<M>) -> LenientIter<'l, M> {
		match self {
			Self::Nil => LenientIter::Nil,
			Self::Cons(_, l, _) => LenientIter::Cons(context, *l),
		}
	}
}

pub struct TryFold<'l, T, M, F> {
	context: &'l Context<M>,
	stack: Vec<Result<(ListRef<'l, M>, T), Error<M>>>,
	f: F
}

impl<'l, T, M, F> Iterator for TryFold<'l, T, M, F> where T: Clone, M: Clone, F: Fn(T, &Single<Stripped<Object<M>>, M>) -> Result<Vec<T>, Error<M>> {
	type Item = Result<T, Error<M>>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.stack.pop() {
				Some(Err(e)) => break Some(Err(e)),
				Some(Ok((list, t))) => {
					match list {
						ListRef::Nil => break Some(Ok(t)),
						ListRef::Cons(id, d, meta) => {
							if d.rest.is_empty() {
								break Some(Err(Meta(
									error::NodeBindingMissing {
										id,
										property: Property::Rest.into()
									}.into(),
									meta.clone()
								)))
							} else {
								match (self.f)(t, &d.first) {
									Ok(new_states) => {
										'next_state: for u in new_states {
											let len = d.rest.len();
		
											for (i, Meta(rest_id, rest_meta)) in d.rest.iter().enumerate() {
												if i + 1 == len {
													let item = match self.context.require_list(*rest_id).map_err(|e| e.at(rest_meta.clone())) {
														Ok(rest) => Ok((rest, u)),
														Err(e) => Err(e)
													};
													
													self.stack.push(item);
													continue 'next_state;
												} else {
													let item = match self.context.require_list(*rest_id).map_err(|e| e.at(rest_meta.clone())) {
														Ok(rest) => Ok((rest, u.clone())),
														Err(e) => Err(e)
													};
													
													self.stack.push(item)
												}
											}
										}
									}
									Err(e) => break Some(Err(e))
								}
							}
						}
					}
				}
				None => break None
			}
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

pub enum ClassBindingRef<'a, M> {
	First(&'a Object<M>),
	Rest(Id),
}

pub type BindingRef<'a, M> = ClassBindingRef<'a, M>;

impl<'a, M> ClassBindingRef<'a, M> {
	pub fn property(&self) -> Property {
		match self {
			Self::First(_) => Property::First,
			Self::Rest(_) => Property::Rest,
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::First(v) => BindingValueRef::Object(v),
			Self::Rest(v) => BindingValueRef::Id(*v)
		}
	}
}

/// Iterator over the bindings of a given list.
pub struct ClassBindings<'a, M> {
	first: single::Iter<'a, Stripped<Object<M>>, M>,
	rest: single::Iter<'a, Id, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.first
			.next()
			.map(|m| m.map(|o| ClassBindingRef::First(&o.0)))
			.or_else(|| {
				self.rest
					.next()
					.map(Meta::into_cloned_value)
					.map(|m| m.map(ClassBindingRef::Rest))
			})
	}
}