use std::cmp::Ordering;

use crate::{
	context::{MapIds, MapIdsIn},
	error, functional_property_value, rdf,
	resource::BindingValueRef,
	Context, Error, FunctionalPropertyValue,
};
use derivative::Derivative;
use locspan::{Meta, Stripped};
use treeldr::{metadata::Merge, vocab::Object, Id, PropertyValueRef};

pub use treeldr::list::Property;

#[derive(Clone)]
pub struct Definition<M> {
	first: FunctionalPropertyValue<Stripped<Object<M>>, M>,
	rest: FunctionalPropertyValue<Id, M>,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			first: FunctionalPropertyValue::default(),
			rest: FunctionalPropertyValue::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn first(&self) -> &FunctionalPropertyValue<Stripped<Object<M>>, M> {
		&self.first
	}

	pub fn first_mut(&mut self) -> &mut FunctionalPropertyValue<Stripped<Object<M>>, M> {
		&mut self.first
	}

	pub fn rest(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.rest
	}

	pub fn rest_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.rest
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(Id, Id) -> Option<Ordering>,
		prop: Property,
		value: Meta<Object<M>, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::First => self.first.insert(None, prop_cmp, value.map(Stripped)),
			Property::Rest => self
				.rest
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
		}

		Ok(())
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			first: self.first.iter(),
			rest: self.rest.iter(),
		}
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.first.map_ids_in(Some(Property::First.into()), &f);
		self.rest.map_ids_in(Some(Property::Rest.into()), f)
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum ListRef<'l, M> {
	Nil,
	Cons(Id, &'l Definition<M>, &'l M),
}

impl<'l, M> ListRef<'l, M> {
	pub fn try_fold<T, F>(&self, context: &'l Context<M>, first: T, f: F) -> TryFold<'l, T, M, F>
	where
		F: Fn(T, &FunctionalPropertyValue<Stripped<Object<M>>, M>) -> Result<Vec<T>, Error<M>>,
	{
		TryFold {
			context,
			stack: vec![Ok((*self, first))],
			f,
		}
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

type TryFoldState<'l, T, M> = (ListRef<'l, M>, T);

pub struct TryFold<'l, T, M, F> {
	context: &'l Context<M>,
	stack: Vec<Result<TryFoldState<'l, T, M>, Error<M>>>,
	f: F,
}

impl<'l, T, M, F> Iterator for TryFold<'l, T, M, F>
where
	T: Clone,
	M: Clone,
	F: Fn(T, &FunctionalPropertyValue<Stripped<Object<M>>, M>) -> Result<Vec<T>, Error<M>>,
{
	type Item = Result<T, Error<M>>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.stack.pop() {
				Some(Err(e)) => break Some(Err(e)),
				Some(Ok((list, t))) => match list {
					ListRef::Nil => break Some(Ok(t)),
					ListRef::Cons(id, d, meta) => {
						if d.rest.is_empty() {
							break Some(Err(Meta(
								error::NodeBindingMissing {
									id,
									property: Property::Rest.into(),
								}
								.into(),
								meta.clone(),
							)));
						} else {
							match (self.f)(t, &d.first) {
								Ok(new_states) => {
									'next_state: for u in new_states {
										let len = d.rest.len();

										for (
											i,
											PropertyValueRef {
												value: Meta(rest_id, rest_meta),
												..
											},
										) in d.rest.iter().enumerate()
										{
											if i + 1 == len {
												let item = match self
													.context
													.require_list(*rest_id)
													.map_err(|e| e.at(rest_meta.clone()))
												{
													Ok(rest) => Ok((rest, u)),
													Err(e) => Err(e),
												};

												self.stack.push(item);
												continue 'next_state;
											} else {
												let item = match self
													.context
													.require_list(*rest_id)
													.map_err(|e| e.at(rest_meta.clone()))
												{
													Ok(rest) => Ok((rest, u.clone())),
													Err(e) => Err(e),
												};

												self.stack.push(item)
											}
										}
									}
								}
								Err(e) => break Some(Err(e)),
							}
						}
					}
				},
				None => break None,
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
				match d
					.first
					.as_required_at_node_binding(*id, Property::First, meta)
				{
					Ok(item) => {
						match d
							.rest
							.as_required_at_node_binding(*id, Property::Rest, meta)
						{
							Ok(rest_id) => {
								match nodes.require_list(**rest_id.value()) {
									Ok(ListRef::Cons(rest_id, rest, rest_meta)) => {
										*self = Self::Cons(*nodes, rest_id, rest, rest_meta)
									}
									Ok(ListRef::Nil) => *self = Self::Nil,
									Err(e) => {
										return Some(Err(e.at_node_property(
											*id,
											Property::Rest,
											meta.clone(),
										)))
									}
								}

								Some(Ok(item.as_meta_value().cloned().map(Stripped::as_ref)))
							}
							Err(e) => Some(Err(e)),
						}
					}
					Err(e) => Some(Err(e)),
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

					match d.rest.first().and_then(|rest| nodes.get_list(**rest.value)) {
						Some(ListRef::Cons(_, rest, _)) => *self = Self::Cons(*nodes, rest),
						_ => *self = Self::Nil,
					}

					if let Some(item) = item {
						break Some(item.value.map(Stripped::as_ref));
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
	First(Option<Id>, &'a Object<M>),
	Rest(Option<Id>, Id),
}

pub type BindingRef<'a, M> = ClassBindingRef<'a, M>;

impl<'a, M> ClassBindingRef<'a, M> {
	pub fn property(&self) -> Property {
		match self {
			Self::First(_, _) => Property::First,
			Self::Rest(_, _) => Property::Rest,
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::First(_, v) => BindingValueRef::Object(v),
			Self::Rest(_, v) => BindingValueRef::Id(*v),
		}
	}
}

/// Iterator over the bindings of a given list.
pub struct ClassBindings<'a, M> {
	first: functional_property_value::Iter<'a, Stripped<Object<M>>, M>,
	rest: functional_property_value::Iter<'a, Id, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.first
			.next()
			.map(|m| m.into_class_binding(|id, o| ClassBindingRef::First(id, &o.0)))
			.or_else(|| {
				self.rest
					.next()
					.map(|m| m.into_cloned_class_binding(ClassBindingRef::Rest))
			})
	}
}
