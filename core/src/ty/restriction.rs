use crate::{
	metadata::Merge,
	multiple,
	node::BindingValueRef,
	prop::{PropertyName, UnknownProperty},
	property_values, vocab, Id, IriIndex, MetaOption, Multiple, RequiredFunctionalPropertyValue,
	TId, Type,
};
use derivative::Derivative;
use locspan::Meta;
use xsd_types::NonNegativeInteger;

/// Property restriction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Restriction {
	/// Range restriction.
	Range(Range),

	/// Cardinality restriction.
	Cardinality(Cardinality),
}

impl Restriction {
	pub fn as_binding_ref(&self) -> ClassBindingRef {
		match self {
			Self::Range(r) => r.as_binding_ref(),
			Self::Cardinality(r) => r.as_binding_ref(),
		}
	}
}

/// Property restriction reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestrictionRef<'a> {
	/// Range restriction.
	Range(Range),

	/// Cardinality restriction.
	Cardinality(CardinalityRef<'a>),
}

impl<'a> RestrictionRef<'a> {
	pub fn as_binding_ref(&self) -> ClassBindingRef<'a> {
		match self {
			Self::Range(r) => r.as_binding_ref(),
			Self::Cardinality(r) => r.as_binding_ref(),
		}
	}
}

/// Property range restriction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Range {
	/// At least one value must be an instance of the given type.
	Any(TId<Type>),

	/// All the values must be instances of the given type.
	All(TId<Type>),
}

impl Range {
	pub fn as_binding(&self) -> ClassBinding {
		match self {
			Self::Any(v) => ClassBinding::SomeValuesFrom(None, *v),
			Self::All(v) => ClassBinding::AllValuesFrom(None, *v),
		}
	}

	pub fn as_binding_ref<'a>(&self) -> ClassBindingRef<'a> {
		match self {
			Self::Any(v) => ClassBindingRef::SomeValuesFrom(None, *v),
			Self::All(v) => ClassBindingRef::AllValuesFrom(None, *v),
		}
	}
}

/// Property cardinality restriction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cardinality {
	/// The property must have at least the given number of values.
	AtLeast(NonNegativeInteger),

	/// The property must have at most the given number of values.
	AtMost(NonNegativeInteger),

	/// The property must have exactly the given number of values.
	Exactly(NonNegativeInteger),
}

impl Cardinality {
	pub fn as_binding_ref(&self) -> ClassBindingRef {
		match self {
			Self::AtLeast(v) => ClassBindingRef::MinCardinality(None, v),
			Self::AtMost(v) => ClassBindingRef::MaxCardinality(None, v),
			Self::Exactly(v) => ClassBindingRef::Cardinality(None, v),
		}
	}
}

/// Property cardinality restriction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CardinalityRef<'a> {
	/// The property must have at least the given number of values.
	AtLeast(&'a NonNegativeInteger),

	/// The property must have at most the given number of values.
	AtMost(&'a NonNegativeInteger),

	/// The property must have exactly the given number of values.
	Exactly(&'a NonNegativeInteger),
}

impl<'a> CardinalityRef<'a> {
	pub fn as_binding_ref(&self) -> ClassBindingRef<'a> {
		match self {
			Self::AtLeast(v) => ClassBindingRef::MinCardinality(None, v),
			Self::AtMost(v) => ClassBindingRef::MaxCardinality(None, v),
			Self::Exactly(v) => ClassBindingRef::Cardinality(None, v),
		}
	}
}

/// Type restricted on a property.
///
/// Puts a restriction on a given property.
/// A restricted type is a subset of the domain of the property which
/// includes every instance for which the given property satisfies the
/// given restriction.
#[derive(Debug)]
pub struct Definition<M> {
	property: RequiredFunctionalPropertyValue<TId<crate::Property>, M>,
	restriction: Meta<Restriction, M>,
}

impl<M> Definition<M> {
	pub fn new(
		property: RequiredFunctionalPropertyValue<TId<crate::Property>, M>,
		restriction: Meta<Restriction, M>,
	) -> Self
	where
		M: Clone + Merge,
	{
		Self {
			property,
			restriction,
		}
	}

	pub fn property(&self) -> TId<crate::Property> {
		*self.property.value()
	}

	pub fn restriction(&self) -> &Meta<Restriction, M> {
		&self.restriction
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			on_property: Some(self.property.iter()),
			restriction: Some(&self.restriction),
		}
	}
}

#[derive(Debug, Clone, Copy)]
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

	pub fn singleton(restriction: Meta<Restriction, M>) -> Self
	where
		M: Clone + Merge,
	{
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

	pub fn restrict(
		&mut self,
		Meta(restriction, meta): Meta<Restriction, M>,
	) -> Result<(), Contradiction>
	where
		M: Clone + Merge,
	{
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

	pub fn union_with(&self, other: &Self) -> Self
	where
		M: Clone + Merge,
	{
		Self {
			range: self.range.union_with(&other.range),
			cardinality: self.cardinality.union_with(&other.cardinality),
		}
	}

	pub fn intersection_with(&self, other: &Self) -> Result<Self, Contradiction>
	where
		M: Clone + Merge,
	{
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
	type Item = Meta<RestrictionRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.range
			.next()
			.map(|r| r.map(RestrictionRef::Range))
			.or_else(|| {
				self.cardinality
					.next()
					.map(|r| r.map(RestrictionRef::Cardinality))
			})
	}
}

impl<'a, M> IntoIterator for &'a Restrictions<M> {
	type Item = Meta<RestrictionRef<'a>, &'a M>;
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

	pub fn restrict(&mut self, Meta(restriction, meta): Meta<Range, M>)
	where
		M: Merge,
	{
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

	pub fn union_with(&self, other: &Self) -> Self
	where
		M: Clone + Merge,
	{
		Self {
			all: self
				.all
				.clone()
				.intersected_with(other.all.iter().map(|m| m.cloned())),
			any: self
				.any
				.clone()
				.intersected_with(other.any.iter().map(|m| m.cloned())),
		}
	}

	pub fn intersection_with(&self, other: &Self) -> Self
	where
		M: Clone + Merge,
	{
		Self {
			all: self
				.all
				.clone()
				.extended_with(other.all.iter().map(|m| m.cloned())),
			any: self
				.any
				.clone()
				.extended_with(other.any.iter().map(|m| m.cloned())),
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
			.or_else(|| self.all.next().map(|Meta(r, m)| Meta(Range::All(*r), m)))
	}
}

#[derive(Debug, Derivative, Clone)]
#[derivative(Default(bound = ""))]
pub struct CardinalityRestrictions<M> {
	min: MetaOption<NonNegativeInteger, M>,
	max: MetaOption<NonNegativeInteger, M>,
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
			min: self.min.as_ref(),
			max: self.max.as_ref(),
		}
	}

	pub fn restrict(
		&mut self,
		Meta(restriction, meta): Meta<Cardinality, M>,
	) -> Result<(), Contradiction>
	where
		M: Clone,
	{
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

				self.min = MetaOption::new(n.clone(), meta.clone());
				self.max = MetaOption::new(n, meta);
			}
		}

		Ok(())
	}

	pub fn clear(&mut self) {
		self.min.clear();
		self.max.clear();
	}

	pub fn union_with(&self, other: &Self) -> Self
	where
		M: Clone,
	{
		let min = match (self.min.as_ref(), other.min.as_ref()) {
			(Some(a), Some(b)) => {
				if **a <= **b {
					Some(a.clone())
				} else {
					Some(b.clone())
				}
			}
			_ => None,
		}
		.into();

		let max = match (self.max.as_ref(), other.max.as_ref()) {
			(Some(a), Some(b)) => {
				if **a >= **b {
					Some(a.clone())
				} else {
					Some(b.clone())
				}
			}
			_ => None,
		}
		.into();

		Self { min, max }
	}

	pub fn intersection_with(&self, other: &Self) -> Result<Self, Contradiction>
	where
		M: Clone,
	{
		let min: MetaOption<NonNegativeInteger, M> =
			match (self.min.as_ref(), other.min.as_ref()) {
				(Some(a), Some(b)) => {
					if **a >= **b {
						Some(a.clone())
					} else {
						Some(b.clone())
					}
				}
				(Some(min), None) => Some(min.clone()),
				(None, Some(min)) => Some(min.clone()),
				(None, None) => None,
			}
			.into();

		let max: MetaOption<NonNegativeInteger, M> =
			match (self.max.as_ref(), other.max.as_ref()) {
				(Some(a), Some(b)) => {
					if **a <= **b {
						Some(a.clone())
					} else {
						Some(b.clone())
					}
				}
				(Some(max), None) => Some(max.clone()),
				(None, Some(max)) => Some(max.clone()),
				_ => None,
			}
			.into();

		if let (Some(min), Some(max)) = (min.value(), max.value()) {
			if min > max {
				return Err(Contradiction);
			}
		}

		Ok(Self { min, max })
	}
}

pub struct CardinalityRestrictionsIter<'a, M> {
	min: Option<&'a Meta<NonNegativeInteger, M>>,
	max: Option<&'a Meta<NonNegativeInteger, M>>,
}

impl<'a, M> Iterator for CardinalityRestrictionsIter<'a, M> {
	type Item = Meta<CardinalityRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.min.map(Meta::value) == self.max.map(Meta::value) {
			self.min.take();
			self.max
				.take()
				.map(|m| m.borrow().map(CardinalityRef::Exactly))
		} else {
			self.min
				.take()
				.map(|m| m.borrow().map(CardinalityRef::AtLeast))
				.or_else(|| {
					self.max
						.take()
						.map(|m| m.borrow().map(CardinalityRef::AtMost))
				})
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	OnProperty(Option<TId<UnknownProperty>>),
	AllValuesFrom(Option<TId<UnknownProperty>>),
	SomeValuesFrom(Option<TId<UnknownProperty>>),
	MinCardinality(Option<TId<UnknownProperty>>),
	MaxCardinality(Option<TId<UnknownProperty>>),
	Cardinality(Option<TId<UnknownProperty>>),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::{Owl, Term};
		match self {
			Self::OnProperty(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::OnProperty))),
			Self::OnProperty(Some(p)) => p.id(),
			Self::AllValuesFrom(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::AllValuesFrom))),
			Self::AllValuesFrom(Some(p)) => p.id(),
			Self::SomeValuesFrom(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::SomeValuesFrom))),
			Self::SomeValuesFrom(Some(p)) => p.id(),
			Self::MinCardinality(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::MinCardinality))),
			Self::MinCardinality(Some(p)) => p.id(),
			Self::MaxCardinality(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::MaxCardinality))),
			Self::MaxCardinality(Some(p)) => p.id(),
			Self::Cardinality(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::Cardinality))),
			Self::Cardinality(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::{Owl, Term};
		match self {
			Self::OnProperty(None) => Some(Term::Owl(Owl::OnProperty)),
			Self::AllValuesFrom(None) => Some(Term::Owl(Owl::AllValuesFrom)),
			Self::SomeValuesFrom(None) => Some(Term::Owl(Owl::SomeValuesFrom)),
			Self::MinCardinality(None) => Some(Term::Owl(Owl::MinCardinality)),
			Self::MaxCardinality(None) => Some(Term::Owl(Owl::MaxCardinality)),
			Self::Cardinality(None) => Some(Term::Owl(Owl::Cardinality)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::OnProperty(None) => PropertyName::Resource("restricted property"),
			Self::OnProperty(Some(p)) => PropertyName::Other(*p),
			Self::AllValuesFrom(None) => PropertyName::Resource("all values from range"),
			Self::AllValuesFrom(Some(p)) => PropertyName::Other(*p),
			Self::SomeValuesFrom(None) => PropertyName::Resource("some values from range"),
			Self::SomeValuesFrom(Some(p)) => PropertyName::Other(*p),
			Self::MinCardinality(None) => PropertyName::Resource("minimum cardinality"),
			Self::MinCardinality(Some(p)) => PropertyName::Other(*p),
			Self::MaxCardinality(None) => PropertyName::Resource("maximum cardinality"),
			Self::MaxCardinality(Some(p)) => PropertyName::Other(*p),
			Self::Cardinality(None) => PropertyName::Resource("cardinality"),
			Self::Cardinality(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_type(&self) -> bool {
		matches!(self, Self::AllValuesFrom(_) | Self::SomeValuesFrom(_))
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}

pub enum ClassBinding {
	OnProperty(Option<TId<UnknownProperty>>, TId<crate::Property>),
	SomeValuesFrom(Option<TId<UnknownProperty>>, TId<Type>),
	AllValuesFrom(Option<TId<UnknownProperty>>, TId<Type>),
	MinCardinality(Option<TId<UnknownProperty>>, NonNegativeInteger),
	MaxCardinality(Option<TId<UnknownProperty>>, NonNegativeInteger),
	Cardinality(Option<TId<UnknownProperty>>, NonNegativeInteger),
}

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::OnProperty(p, _) => Property::OnProperty(*p),
			Self::SomeValuesFrom(p, _) => Property::SomeValuesFrom(*p),
			Self::AllValuesFrom(p, _) => Property::AllValuesFrom(*p),
			Self::MinCardinality(p, _) => Property::MinCardinality(*p),
			Self::MaxCardinality(p, _) => Property::MaxCardinality(*p),
			Self::Cardinality(p, _) => Property::Cardinality(*p),
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<M> {
		match self {
			Self::OnProperty(_, v) => BindingValueRef::Property(*v),
			Self::SomeValuesFrom(_, v) => BindingValueRef::Type(*v),
			Self::AllValuesFrom(_, v) => BindingValueRef::Type(*v),
			Self::MinCardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
			Self::MaxCardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
			Self::Cardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
		}
	}
}

pub enum ClassBindingRef<'a> {
	OnProperty(Option<TId<UnknownProperty>>, TId<crate::Property>),
	SomeValuesFrom(Option<TId<UnknownProperty>>, TId<Type>),
	AllValuesFrom(Option<TId<UnknownProperty>>, TId<Type>),
	MinCardinality(Option<TId<UnknownProperty>>, &'a NonNegativeInteger),
	MaxCardinality(Option<TId<UnknownProperty>>, &'a NonNegativeInteger),
	Cardinality(Option<TId<UnknownProperty>>, &'a NonNegativeInteger),
}

impl<'a> ClassBindingRef<'a> {
	pub fn property(&self) -> Property {
		match self {
			Self::OnProperty(p, _) => Property::OnProperty(*p),
			Self::SomeValuesFrom(p, _) => Property::SomeValuesFrom(*p),
			Self::AllValuesFrom(p, _) => Property::AllValuesFrom(*p),
			Self::MinCardinality(p, _) => Property::MinCardinality(*p),
			Self::MaxCardinality(p, _) => Property::MaxCardinality(*p),
			Self::Cardinality(p, _) => Property::Cardinality(*p),
		}
	}

	pub fn value<M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::OnProperty(_, v) => BindingValueRef::Property(*v),
			Self::SomeValuesFrom(_, v) => BindingValueRef::Type(*v),
			Self::AllValuesFrom(_, v) => BindingValueRef::Type(*v),
			Self::MinCardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
			Self::MaxCardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
			Self::Cardinality(_, v) => BindingValueRef::NonNegativeInteger(v),
		}
	}
}

pub type Binding = ClassBinding;

pub type BindingRef<'a> = ClassBindingRef<'a>;

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	on_property: Option<property_values::required_functional::Iter<'a, TId<crate::Property>, M>>,
	restriction: Option<&'a Meta<Restriction, M>>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_property
			.as_mut()
			.and_then(|i| {
				i.next()
					.map(|m| m.into_cloned_class_binding(ClassBindingRef::OnProperty))
			})
			.or_else(|| {
				self.restriction
					.take()
					.map(|m| m.borrow().map(Restriction::as_binding_ref))
			})
	}
}
