use super::Properties;
use crate::{TId, metadata::Merge, vocab, Multiple, Type, multiple, MetaOption};
use derivative::Derivative;
use locspan::Meta;

/// Property restriction.
#[derive(Derivative)]
#[derivative(
	Clone(bound = ""),
	Copy(bound = ""),
	PartialEq(bound = ""),
	Eq(bound = "")
)]
pub enum Restriction {
	/// Range restriction.
	Range(Range),

	/// Cardinality restriction.
	Cardinality(Cardinality),
}

/// Property range restriction.
#[derive(Derivative)]
#[derivative(
	Clone(bound = ""),
	Copy(bound = ""),
	PartialEq(bound = ""),
	Eq(bound = "")
)]
pub enum Range {
	/// At least one value must be an instance of the given type.
	Any(TId<Type>),

	/// All the values must be instances of the given type.
	All(TId<Type>),
}

/// Property cardinality restriction.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cardinality {
	/// The property must have at least the given number of values.
	AtLeast(u32),

	/// The property must have at most the given number of values.
	AtMost(u32),

	/// The property must have exactly the given number of values.
	Exactly(u32),
}

/// Type restricted on a property.
///
/// Puts a restriction on a given property.
/// A restricted type is a subset of the domain of the property which
/// includes every instance for which the given property satisfies the
/// given restriction.
#[derive(Debug)]
pub struct Definition<M> {
	properties: Properties<M>,
}

impl<M> Definition<M> {
	pub fn new(
		Meta(prop, causes): Meta<TId<crate::Property>, M>,
		restriction: Meta<Restriction, M>
	) -> Self
	where
		M: Clone + Merge,
	{
		let mut properties = Properties::none();

		properties.insert(prop, Some(Restrictions::singleton(restriction)), causes);

		Self { properties }
	}

	pub fn properties(&self) -> &Properties<M> {
		&self.properties
	}

	pub fn property(&self) -> TId<crate::Property> {
		self.properties.included().next().unwrap().property()
	}

	pub fn restrictions(&self) -> &Restrictions<M> {
		self.properties.included().next().unwrap().restrictions()
	}
}

#[derive(Clone, Copy)]
pub struct Contradiction;

#[derive(Debug, Derivative, Clone)]
#[derivative(Default(bound = ""))]
pub struct Restrictions<M> {
	range: RangeRestrictions<M>,
	cardinality: CardinalityRestrictions<M>,
}

impl<M> Restrictions<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn singleton(restriction: Meta<Restriction, M>) -> Self where M: Clone + Merge {
		let mut result = Self::new();
		result.restrict(restriction).ok().unwrap();
		result
	}

	pub fn len(&self) -> usize {
		self.range.len() + self.cardinality.len()
	}

	pub fn is_empty(&self) -> bool {
		self.range.is_empty() && self.cardinality.is_empty()
	}

	pub fn iter(&self) -> Iter<M> {
		Iter {
			range: self.range.iter(),
			cardinality: self.cardinality.iter(),
		}
	}

	pub fn restrict(&mut self, Meta(restriction, meta): Meta<Restriction, M>) -> Result<(), Contradiction> where M: Clone + Merge {
		match restriction {
			Restriction::Range(r) => {
				self.range.restrict(Meta(r, meta));
				Ok(())
			}
			Restriction::Cardinality(c) => self.cardinality.restrict(Meta(c, meta)),
		}
	}

	pub fn clear(&mut self) {
		self.range.clear();
		self.cardinality.clear()
	}

	pub fn union_with(&self, other: &Self) -> Self where M: Clone + Merge {
		Self {
			range: self.range.union_with(&other.range),
			cardinality: self.cardinality.union_with(&other.cardinality),
		}
	}

	pub fn intersection_with(&self, other: &Self) -> Result<Self, Contradiction> where M: Clone + Merge {
		Ok(Self {
			range: self.range.intersection_with(&other.range),
			cardinality: self.cardinality.intersection_with(&other.cardinality)?,
		})
	}
}

pub struct Iter<'a, M> {
	range: RangeRestrictionsIter<'a, M>,
	cardinality: CardinalityRestrictionsIter<'a, M>,
}

impl<'a, M> Iterator for Iter<'a, M> {
	type Item = Meta<Restriction, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.range
			.next()
			.map(|r| r.map(Restriction::Range))
			.or_else(|| {
				self.cardinality
					.next()
					.map(|r| r.map(Restriction::Cardinality))
			})
	}
}

impl<'a, M> IntoIterator for &'a Restrictions<M> {
	type Item = Meta<Restriction, &'a M>;
	type IntoIter = Iter<'a, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

#[derive(Debug, Derivative, Clone)]
#[derivative(Default(bound = ""))]
pub struct RangeRestrictions<M> {
	all: Multiple<TId<Type>, M>,
	any: Multiple<TId<Type>, M>,
}

impl<M> RangeRestrictions<M> {
	pub fn len(&self) -> usize {
		self.all.len() + self.any.len()
	}

	pub fn is_empty(&self) -> bool {
		self.all.is_empty() && self.any.is_empty()
	}

	pub fn iter(&self) -> RangeRestrictionsIter<M> {
		RangeRestrictionsIter {
			all: self.all.iter(),
			any: self.any.iter(),
		}
	}

	pub fn restrict(&mut self, Meta(restriction, meta): Meta<Range, M>) where M: Merge {
		match restriction {
			Range::All(r) => {
				self.all.insert(Meta(r, meta));
			}
			Range::Any(r) => {
				self.any.insert(Meta(r, meta));
			}
		}
	}

	pub fn clear(&mut self) {
		self.all.clear();
		self.any.clear();
	}

	pub fn union_with(&self, other: &Self) -> Self where M: Clone + Merge {
		Self {
			all: self.all.clone().intersected_with(other.all.iter().map(|m| m.cloned())),
			any: self.any.clone().intersected_with(other.any.iter().map(|m| m.cloned())),
		}
	}

	pub fn intersection_with(&self, other: &Self) -> Self where M: Clone + Merge {
		Self {
			all: self.all.clone().extended_with(other.all.iter().map(|m| m.cloned())),
			any: self.any.clone().extended_with(other.any.iter().map(|m| m.cloned())),
		}
	}
}

pub struct RangeRestrictionsIter<'a, M> {
	all: multiple::Iter<'a, TId<Type>, M>,
	any: multiple::Iter<'a, TId<Type>, M>,
}

impl<'a, M> Iterator for RangeRestrictionsIter<'a, M> {
	type Item = Meta<Range, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.any
			.next()
			.map(|Meta(r, m)| Meta(Range::Any(*r), m))
			.or_else(|| {
				self.all
					.next()
					.map(|Meta(r, m)| Meta(Range::All(*r), m))
			})
	}
}

#[derive(Debug, Derivative, Clone)]
#[derivative(Default(bound = ""))]
pub struct CardinalityRestrictions<M> {
	min: MetaOption<u32, M>,
	max: MetaOption<u32, M>,
}

impl<M> CardinalityRestrictions<M> {
	pub fn len(&self) -> usize {
		match (self.min.value(), self.max.value()) {
			(Some(min), Some(max)) if min == max => 1,
			(Some(_), Some(_)) => 2,
			(Some(_), None) => 1,
			(None, Some(_)) => 1,
			(None, None) => 0,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.min.is_none() && self.max.is_none()
	}

	pub fn iter(&self) -> CardinalityRestrictionsIter<M> {
		CardinalityRestrictionsIter {
			min: self.min.as_ref().map(Meta::borrow_metadata),
			max: self.max.as_ref().map(Meta::borrow_metadata),
		}
	}

	pub fn restrict(&mut self, Meta(restriction, meta): Meta<Cardinality, M>) -> Result<(), Contradiction> where M: Clone {
		match restriction {
			Cardinality::AtLeast(min) => {
				if let Some(max) = self.max.value() {
					if min > *max {
						return Err(Contradiction);
					}
				}

				self.min = MetaOption::new(min, meta)
			}
			Cardinality::AtMost(max) => {
				if let Some(min) = self.min.value() {
					if *min > max {
						return Err(Contradiction);
					}
				}

				self.max = MetaOption::new(max, meta)
			}
			Cardinality::Exactly(n) => {
				if let Some(min) = self.min.value() {
					if *min > n {
						return Err(Contradiction);
					}
				}

				if let Some(max) = self.max.value() {
					if n > *max {
						return Err(Contradiction);
					}
				}

				self.min = MetaOption::new(n, meta.clone());
				self.max = MetaOption::new(n, meta);
			}
		}

		Ok(())
	}

	pub fn clear(&mut self) {
		self.min.clear();
		self.max.clear();
	}

	pub fn union_with(&self, other: &Self) -> Self where M: Clone {
		let min = match (self.min.as_ref(), other.min.as_ref()) {
			(Some(a), Some(b)) => {
				if **a <= **b {
					Some(a.clone())
				} else {
					Some(b.clone())
				}
			},
			_ => None,
		}.into();

		let max = match (self.max.as_ref(), other.max.as_ref()) {
			(Some(a), Some(b)) => {
				if **a >= **b {
					Some(a.clone())
				} else {
					Some(b.clone())
				}
			},
			_ => None,
		}.into();

		Self { min, max }
	}

	pub fn intersection_with(&self, other: &Self) -> Result<Self, Contradiction> where M: Clone {
		let min: MetaOption<u32, M> = match (self.min.as_ref(), other.min.as_ref()) {
			(Some(a), Some(b)) => {
				if **a >= **b {
					Some(a.clone())
				} else {
					Some(b.clone())
				}
			},
			(Some(min), None) => Some(min.clone()),
			(None, Some(min)) => Some(min.clone()),
			(None, None) => None,
		}.into();

		let max: MetaOption<u32, M> = match (self.max.as_ref(), other.max.as_ref()) {
			(Some(a), Some(b)) => {
				if **a <= **b {
					Some(a.clone())
				} else {
					Some(b.clone())
				}
			},
			(Some(max), None) => Some(max.clone()),
			(None, Some(max)) => Some(max.clone()),
			_ => None,
		}.into();

		if let (Some(min), Some(max)) = (min.value(), max.value()) {
			if min > max {
				return Err(Contradiction);
			}
		}

		Ok(Self { min, max })
	}
}

pub struct CardinalityRestrictionsIter<'a, M> {
	min: Option<Meta<u32, &'a M>>,
	max: Option<Meta<u32, &'a M>>,
}

impl<'a, M> Iterator for CardinalityRestrictionsIter<'a, M> {
	type Item = Meta<Cardinality, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.min.as_deref() == self.max.as_deref() {
			self.min.take();
			self.max.take().map(|m| m.map(Cardinality::Exactly))
		} else {
			self.min
				.take()
				.map(|m| m.map(Cardinality::AtLeast))
				.or_else(|| {
					self.max.take().map(|m| m.map(Cardinality::AtMost))
				})
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	OnProperty,
	AllValuesFrom,
	SomeValuesFrom
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Term, Owl};
		match self {
			Self::OnProperty => Term::Owl(Owl::OnProperty),
			Self::AllValuesFrom => Term::Owl(Owl::AllValuesFrom),
			Self::SomeValuesFrom => Term::Owl(Owl::SomeValuesFrom),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::OnProperty => "restricted property",
			Self::AllValuesFrom => "all values from range",
			Self::SomeValuesFrom => "some values from range"
		}
	}
}