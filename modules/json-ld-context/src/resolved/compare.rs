use json_ld::{
	context::Nest, syntax::context::term_definition::Index, Direction, LenientLanguageTagBuf,
	Nullable,
};
use shelves::Ref;
use std::{
	cell::RefCell,
	cmp::Ordering,
	collections::{hash_map, HashMap},
};
use treeldr::{BlankIdIndex, IriIndex};

use super::{AccessibleBindings, LocalContext, LocalContexts, TermDefinition};

pub trait CompareThen {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering>;
}

impl CompareThen for json_ld::Term<IriIndex, BlankIdIndex> {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		if self == other {
			f(ord)
		} else {
			None
		}
	}
}

impl CompareThen for json_ld::Type<IriIndex> {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		if self == other {
			f(ord)
		} else {
			None
		}
	}
}

impl CompareThen for json_ld::Container {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		if self == other {
			f(ord)
		} else {
			None
		}
	}
}

impl CompareThen for LenientLanguageTagBuf {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		if self == other {
			f(ord)
		} else {
			None
		}
	}
}

impl CompareThen for Direction {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		if self == other {
			f(ord)
		} else {
			None
		}
	}
}

impl CompareThen for bool {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		if self == other {
			f(ord)
		} else {
			None
		}
	}
}

impl CompareThen for Index {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		if self == other {
			f(ord)
		} else {
			None
		}
	}
}

impl CompareThen for Nest {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		if self == other {
			f(ord)
		} else {
			None
		}
	}
}

impl<T: CompareThen> CompareThen for Nullable<T> {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		match (self, other) {
			(Self::Null, Self::Null) => f(ord),
			(Self::Some(a), Self::Some(b)) => a.compare_then(b, ord, f),
			_ => None,
		}
	}
}

impl<T: CompareThen> CompareThen for Option<T> {
	fn compare_then(
		&self,
		other: &Self,
		ord: Ordering,
		f: impl FnOnce(Ordering) -> Option<Ordering>,
	) -> Option<Ordering> {
		match (self, other) {
			(Some(a), Some(b)) => a.compare_then(b, ord, f),
			(Some(_), None) if ord.is_ge() => f(Ordering::Greater),
			(None, Some(_)) if ord.is_le() => f(Ordering::Less),
			(None, None) => f(ord),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermDefinitionOrdering {
	pub ordering: Option<Ordering>,
	pub header_ordering: Option<Ordering>,
	pub context_ordering: Option<Ordering>,
}

pub trait CompareTermDefinition {
	fn compare(
		&self,
		context_comparison: &LocalContextComparison,
		other: &Self,
	) -> TermDefinitionOrdering;
}

impl CompareTermDefinition for TermDefinition {
	fn compare(
		&self,
		context_comparison: &LocalContextComparison,
		other: &Self,
	) -> TermDefinitionOrdering {
		let mut context_ordering = None;
		let mut header_ordering = None;
		let ordering = self.id.compare_then(&other.id, Ordering::Equal, |ord| {
			self.type_.compare_then(&other.type_, ord, |ord| {
				self.container.compare_then(&other.container, ord, |ord| {
					self.index.compare_then(&other.index, ord, |ord| {
						self.language.compare_then(&other.language, ord, |ord| {
							self.direction.compare_then(&other.direction, ord, |ord| {
								self.prefix.compare_then(&other.prefix, ord, |ord| {
									self.reverse.compare_then(&other.reverse, ord, |ord| {
										self.nest.compare_then(&other.nest, ord, |a| {
											header_ordering = Some(ord);
											context_ordering = context_comparison
												.compare(self.context, other.context);
											match (a, context_ordering) {
												(Ordering::Equal, Some(Ordering::Equal)) => {
													Some(Ordering::Equal)
												}
												(
													Ordering::Greater | Ordering::Equal,
													Some(Ordering::Greater | Ordering::Equal),
												) => Some(Ordering::Greater),
												(
													Ordering::Less | Ordering::Equal,
													Some(Ordering::Less | Ordering::Equal),
												) => Some(Ordering::Less),
												_ => None,
											}
										})
									})
								})
							})
						})
					})
				})
			})
		});

		TermDefinitionOrdering {
			ordering,
			header_ordering,
			context_ordering,
		}
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalContextPair(Ref<LocalContext>, Ref<LocalContext>);

impl LocalContextPair {
	pub fn new(a: Ref<LocalContext>, b: Ref<LocalContext>) -> Self {
		if a.index() > b.index() {
			Self(b, a)
		} else {
			Self(a, b)
		}
	}
}

pub struct LocalContextComparison<'a> {
	local_contexts: &'a LocalContexts,
	accessible_bindings: HashMap<Ref<LocalContext>, AccessibleBindings<'a>>,
	state: RefCell<HashMap<LocalContextPair, Option<Ordering>>>,
}

impl<'a> LocalContextComparison<'a> {
	pub fn new(
		local_contexts: &'a LocalContexts,
		accessible_bindings: HashMap<Ref<LocalContext>, AccessibleBindings<'a>>,
	) -> Self {
		Self {
			local_contexts,
			accessible_bindings,
			state: RefCell::new(HashMap::new()),
		}
	}

	pub fn accessible_bindings(&self, r: Ref<LocalContext>) -> Option<&AccessibleBindings<'a>> {
		self.accessible_bindings.get(&r)
	}

	fn begin(
		&self,
		a: Ref<LocalContext>,
		b: Ref<LocalContext>,
		hint: Ordering,
	) -> Option<Option<Ordering>> {
		let mut map = self.state.borrow_mut();
		match map.entry(LocalContextPair::new(a, b)) {
			hash_map::Entry::Occupied(e) => Some(*e.get()),
			hash_map::Entry::Vacant(e) => {
				e.insert(Some(hint));
				None
			}
		}
	}

	fn end(
		&self,
		a: Ref<LocalContext>,
		b: Ref<LocalContext>,
		ord: Option<Ordering>,
	) -> Option<Ordering> {
		let mut map = self.state.borrow_mut();
		map.insert(LocalContextPair::new(a, b), ord);
		ord
	}

	pub fn compare(&self, a: Ref<LocalContext>, b: Ref<LocalContext>) -> Option<Ordering> {
		let a_bindings = self.local_contexts.definitions.get(&a).unwrap();
		let b_bindings = self.local_contexts.definitions.get(&b).unwrap();

		let mut hint = a_bindings.len().cmp(&b_bindings.len());

		match self.begin(a, b, hint) {
			Some(cmp) => cmp,
			None => match hint {
				Ordering::Less | Ordering::Equal => {
					for (t, a_defs) in a_bindings {
						match b_bindings.get(t) {
							Some(b_defs) => {
								for a_def in &a_defs.added {
									for b_def in &b_defs.added {
										match a_def.compare(self, b_def).ordering {
											Some(Ordering::Greater | Ordering::Equal) => (),
											Some(Ordering::Less) => hint = Ordering::Less,
											_ => return self.end(a, b, None),
										}
									}
								}
							}
							None => return self.end(a, b, None),
						}
					}

					Some(hint)
				}
				Ordering::Greater => {
					for (t, b_defs) in b_bindings {
						match a_bindings.get(t) {
							Some(a_defs) => {
								for a_def in &a_defs.added {
									for b_def in &b_defs.added {
										match a_def.compare(self, b_def).ordering {
											Some(ord) if ord.is_ge() => (),
											_ => return self.end(a, b, None),
										}
									}
								}
							}
							None => return self.end(a, b, None),
						}
					}

					Some(Ordering::Greater)
				}
			},
		}
	}
}
