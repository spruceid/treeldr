use crate::{utils::{self, take_first, Ignored}, value, Domain, Literal, Value};
use educe::Educe;
use iref::{iri, Iri};
use iregex::automata::{AnyRange, Automaton, RangeSet};
use std::{
	cmp::Ordering, collections::{BTreeMap, BTreeSet}, hash::Hash, ops::Bound, sync::OnceLock
};

mod r#ref;
pub use r#ref::*;

pub const TYPE_ANY: &Iri = iri!("http://schema.treeldr.org/type#any");
pub const TYPE_UNIT: &Iri = iri!("http://schema.treeldr.org/type#unit");
pub const TYPE_BOOLEAN: &Iri = iri!("http://schema.treeldr.org/type#bool");
pub const TYPE_NUMBER: &Iri = iri!("http://schema.treeldr.org/type#number");
pub const TYPE_BYTE_STRING: &Iri = iri!("http://schema.treeldr.org/type#bytes");
pub const TYPE_TEXT_STRING: &Iri = iri!("http://schema.treeldr.org/type#string");

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralType {
	Unit,
	Boolean(Boolean),
	Number(Number),
	ByteString(ByteString),
	TextString(TextString),
}

impl LiteralType {
	pub fn contains(&self, value: &Literal) -> bool {
		match (self, value) {
			(Self::Unit, Literal::Unit) => true,
			(Self::Boolean(ty), Literal::Boolean(b)) => ty.contains(*b),
			(Self::Number(ty), Literal::Number(n)) => ty.contains(n),
			(Self::ByteString(ty), Literal::ByteString(b)) => ty.contains(b),
			(Self::TextString(ty), Literal::TextString(t)) => ty.contains(t),
			_ => false,
		}
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(Self::Unit, Self::Unit) => Some(Ordering::Equal),
			(Self::Boolean(a), Self::Boolean(b)) => a.subtype_cmp(b),
			(Self::Number(a), Self::Number(b)) => a.subtype_cmp(b),
			(Self::ByteString(a), Self::ByteString(b)) => a.subtype_cmp(b),
			(Self::TextString(a), Self::TextString(b)) => a.subtype_cmp(b),
			_ => None
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd, T: PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord, T: Ord"))]
pub enum Type<R, T = TypeRef<R>> {
	/// RDF resource.
	Resource(Resource<R>),

	/// Literal value.
	Literal(LiteralType),

	/// Struct.
	Struct(Struct<T>),

	/// Enum.
	Enum(Enum<T>),

	/// List.
	List(List<T>),

	/// Any.
	Any,
}

impl<R, T> Type<R, T> {
	pub fn resource() -> Self {
		Self::Resource(Resource::default())
	}

	pub fn is_resource(&self) -> bool {
		matches!(self, Self::Resource(_))
	}

	pub fn as_list(&self) -> Option<&List<T>> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}
}

impl<R> Type<R> {
	pub fn into_ref(self) -> TypeRef<R>
	where
		R: 'static + Eq + Hash,
	{
		TypeRef::new(self)
	}
}

impl<R: Ord> Type<R> {
	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> where R: 'static {
		match (self, other) {
			(Self::Any, Self::Any) => Some(Ordering::Equal),
			(Self::Any, _) => Some(Ordering::Greater),
			(_, Self::Any) => Some(Ordering::Less),
			(Self::Resource(a), Self::Resource(b)) => a.subtype_cmp(b),
			(Self::Literal(a), Self::Literal(b)) => a.subtype_cmp(b),
			(Self::List(a), Self::List(b)) => a.subtype_cmp(b),
			(Self::Struct(a), Self::Struct(b)) => a.subtype_cmp(b),
			(Self::Enum(a), b) => a.subtype_cmp(b),
			(a, Self::Enum(b)) => b.subtype_cmp(a).map(Ordering::reverse),
			_ => None
		}
	}
	
	pub fn contains(&self, value: &Value<R>) -> bool {
		match (self, value) {
			(Self::Any, _) => true,
			(Self::Resource(ty), Value::Resource(r)) => ty.contains(r),
			(Self::Literal(ty), Value::Literal(l)) => ty.contains(l),
			(Self::List(ty), Value::List(list)) => ty.contains(list),
			(Self::Struct(ty), Value::Map(map)) => ty.contains(map),
			(Self::Enum(ty), value) => ty.contains(value),
			_ => false,
		}
	}
}

/// Resource type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(Default)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord"))]
pub struct Resource<R> {
	set: Option<BTreeSet<R>>
}

impl<R: Ord> Resource<R> {
	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (&self.set, &other.set) {
			(Some(a), Some(b)) => {
				if a == b {
					Some(Ordering::Equal)
				} else if a.is_superset(b) {
					Some(Ordering::Greater)
				} else if a.is_subset(b) {
					Some(Ordering::Less)
				} else {
					None
				}
			}
			(None, Some(_)) => Some(Ordering::Greater),
			(Some(_), None) => Some(Ordering::Less),
			(None, None) => Some(Ordering::Equal)
		}
	}

	pub fn contains(&self, resource: &R) -> bool {
		match &self.set {
			Some(set) => set.contains(resource),
			None => true
		}
	}
}

impl<R> Domain for Resource<R> {
	type Resource = R;
}

/// Struct type.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Struct<T> {
	/// Fields definitions.
	///
	/// # Soundness
	///
	/// It is a logical error for two keys to have a non-null intersection.
	pub fields: BTreeMap<T, Field<T>>,
}

impl<R: Ord> Struct<TypeRef<R>> {
	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> where R: 'static {
		if self.is_subtype_of(other) {
			if other.is_subtype_of(self) {
				Some(Ordering::Equal)
			} else {
				Some(Ordering::Less)
			}
		} else if other.is_subtype_of(self) {
			Some(Ordering::Greater)
		} else {
			None
		}
	}

	pub fn is_subtype_of(&self, other: &Self) -> bool where R: 'static {
		let mut fields: Vec<_> = other.fields.iter().collect();

		for (key, field) in &self.fields {
			let other_field = utils::take_first(&mut fields, |(other_key, _)| {
				key.is_subtype_of(other_key)
			}).or_else(|| {
				other.fields.iter().find(|(other_key, _)| {
					key.is_subtype_of(other_key)
				})
			});

			match other_field {
				Some((_, other_field)) => {
					if !field.is_subfield_of(other_field) {
						return false
					}
				}
				None => {
					return false
				}
			}
		}

		fields.into_iter().all(|(_, field)| !field.required)
	}

	pub fn contains(&self, map: &BTreeMap<Value<R>, Value<R>>) -> bool {
		let mut fields: Vec<_> = self.fields.iter().collect();

		for (key, _) in map {
			if take_first(&mut fields, |(ty, _)| ty.contains(key)).is_none() {
				if !self.fields.keys().any(|ty| ty.contains(key)) {
					return false; // unexpected field
				}
			}
		}

		for (_, field) in fields {
			if field.required {
				return false; // missing required field
			}
		}

		true
	}
}

impl<R> Domain for Struct<TypeRef<R>> {
	type Resource = R;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Field<T> {
	pub type_: T,
	pub required: bool,
}

impl<R: Ord + 'static> Field<TypeRef<R>> {
	pub fn is_subfield_of(&self, other: &Self) -> bool {
		(!other.required || self.required) && self.type_.is_subtype_of(&other.type_)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enum<T> {
	pub variants: BTreeMap<String, T>,
}

impl<R: Ord> Enum<TypeRef<R>> {
	pub fn subtype_cmp(&self, other: &Type<R>) -> Option<Ordering> where R: 'static {
		match other {
			Type::Enum(other) => {
				let mut cmp = Ordering::Equal;

				for a in self.variants.values() {
					for b in other.variants.values() {
						cmp = refine_cmp(cmp, a.subtype_cmp(b))?
					}
				}

				Some(cmp)
			}
			other => {
				let mut cmp = None;
				let mut leq = true;
				
				for (_, ty) in &self.variants {
					match (cmp, ty.as_ref().subtype_cmp(other)) {
						(None | Some(Ordering::Greater | Ordering::Equal), Some(Ordering::Greater)) => {
							leq = false;
							cmp = Some(Ordering::Greater)
						}
						(None | Some(Ordering::Equal), Some(Ordering::Equal)) => {
							cmp = Some(Ordering::Equal)
						}
						(_, None) => {
							leq = false
						}
						_ => ()
					}
				}

				if leq {
					Some(Ordering::Less)
				} else {
					cmp
				}
			}
		}
	}

	pub fn contains(&self, value: &Value<R>) -> bool {
		self.variants.values().any(|ty| ty.contains(value))
	}
}

impl<R> Domain for Enum<R> {
	type Resource = R;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct List<T> {
	pub prefix: Vec<T>,
	pub rest: Option<T>
}

impl<T> List<T> {
	pub fn from_prefix(prefix: Vec<T>) -> Self {
		Self {
			prefix,
			rest: None
		}
	}

	pub fn uniform(ty: T) -> Self {
		Self {
			prefix: Vec::new(),
			rest: Some(ty)
		}
	}
}

impl<R> List<TypeRef<R>> {
	pub fn item_type(&self, i: usize) -> Option<&TypeRef<R>> {
		if i < self.prefix.len() {
			Some(&self.prefix[i])
		} else {
			self.rest.as_ref()
		}
	}

	pub fn min_len(&self) -> usize {
		self.prefix.len()
	}

	pub fn max_len(&self) -> Option<usize> {
		if self.rest.is_some() {
			None
		} else {
			Some(self.prefix.len())
		}
	}

	pub fn as_uniform(&self) -> Option<&TypeRef<R>> {
		if self.prefix.is_empty() {
			self.rest.as_ref()
		} else {
			None
		}
	}
}

impl<R: Ord> List<TypeRef<R>> {
	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> where R: 'static {
		let mut a_prefix = self.prefix.iter();
		let mut b_prefix = other.prefix.iter();

		let mut cmp = Ordering::Equal;

		loop {
			cmp = match (a_prefix.next(), b_prefix.next()) {
				(Some(a), Some(b)) => {
					refine_cmp(cmp, a.subtype_cmp(b))
				},
				(Some(a), None) => {
					refine_cmp(cmp, a.subtype_cmp(other.rest.as_ref()?))
				}
				(None, Some(b)) => {
					refine_cmp(cmp, self.rest.as_ref()?.subtype_cmp(b))
				}
				(None, None) => break
			}?
		}

		let rest_cmp = match (&self.rest, &other.rest) {
			(Some(a), Some(b)) => a.subtype_cmp(b),
			(Some(_), None) => Some(Ordering::Greater),
			(None, Some(_)) => Some(Ordering::Less),
			(None, None) => Some(Ordering::Equal)
		};

		refine_cmp(cmp, rest_cmp)
	}

	pub fn contains(&self, value: &[Value<R>]) -> bool {
		if value.len() < self.min_len() {
			return false
		}

		value.iter().enumerate().all(|(i, item)| self.item_type(i).is_some_and(|ty| ty.contains(item)))
	}
}

fn refine_cmp(cmp: Ordering, refine_with: Option<Ordering>) -> Option<Ordering> {
	match (cmp, refine_with?) {
		(Ordering::Equal, cmp) => Some(cmp),
		(Ordering::Greater, Ordering::Greater | Ordering::Equal) => Some(Ordering::Greater),
		(Ordering::Less, Ordering::Less | Ordering::Equal) => Some(Ordering::Less),
		_ => None
	}
}

impl<R> Domain for List<R> {
	type Resource = R;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Boolean {
	const_: Option<bool>
}

impl Boolean {
	pub fn singleton(b: bool) -> Self {
		Self {
			const_: Some(b)
		}
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self.const_, other.const_) {
			(Some(a), Some(b)) => if a == b {
				Some(Ordering::Equal)
			} else {
				None
			}
			(None, Some(_)) => Some(Ordering::Greater),
			(Some(_), None) => Some(Ordering::Less),
			(None, None) => Some(Ordering::Equal)
		}
	}

	pub fn contains(&self, value: bool) -> bool {
		match self.const_ {
			Some(b) => b == value,
			None => true
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumberDomain {
	/// Rational numbers (ℚ).
	#[default]
	Rational,

	/// Integer numbers (ℤ).
	Integer
}

impl NumberDomain {
	pub fn is_integer(&self) -> bool {
		matches!(self, Self::Integer)
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MinBound {
	#[default]
	Unbounded,
	Excluded(value::Number),
	Included(value::Number)
}

impl MinBound {
	pub fn normalize(&mut self, domain: NumberDomain) {
		if domain.is_integer() {
			if let Self::Excluded(n) = &self {
				*self = Self::Included(n.clone() + 1)
			}
		}
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaxBound {
	Included(value::Number),
	Excluded(value::Number),
	#[default]
	Unbounded,
}

impl MaxBound {
	pub fn normalize(&mut self, domain: NumberDomain) {
		if domain.is_integer() {
			if let Self::Excluded(n) = &self {
				*self = Self::Included(n.clone() - 1)
			}
		}
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Number {
	/// Number domain.
	/// 
	/// NOTE: this is not public because the `min` and `max` bounds are
	/// normalized differently depending on this value.
	domain: NumberDomain,

	/// Min bound.
	/// 
	/// NOTE: this is not public because it is normalized differently depending
	/// on the `domain` value.
	min: MinBound,

	/// Max bound.
	/// 
	/// NOTE: this is not public because it is normalized differently depending
	/// on the `domain` value.
	max: MaxBound
}

impl Number {
	pub fn singleton(value: value::Number) -> Self {
		let domain = if value.is_integer() {
			NumberDomain::Integer
		} else {
			NumberDomain::Rational
		};

		Self {
			domain,
			min: MinBound::Included(value.clone()),
			max: MaxBound::Included(value)
		}
	}

	pub fn as_singleton(&self) -> Option<&value::Number> {
		match (&self.min, &self.max) {
			(MinBound::Included(a), MaxBound::Included(b)) if a == b => Some(a),
			_ => None
		}
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match self.min.cmp(&other.min) {
			Ordering::Less => {
				match self.max.cmp(&other.max) {
					Ordering::Less => {
						None
					}
					Ordering::Greater | Ordering::Equal => {
						match (self.domain, other.domain) {
							(NumberDomain::Rational, _) | (NumberDomain::Integer, NumberDomain::Integer) => {
								Some(Ordering::Greater)
							}
							_ => None
						}
					}
				}
			}
			Ordering::Equal => {
				match self.max.cmp(&other.max) {
					Ordering::Less => {
						match (self.domain, other.domain) {
							(_, NumberDomain::Rational) | (NumberDomain::Integer, NumberDomain::Integer) => {
								Some(Ordering::Less)
							}
							_ => None
						}
					}
					Ordering::Equal => {
						match (self.domain, other.domain) {
							(NumberDomain::Rational, NumberDomain::Rational) | (NumberDomain::Integer, NumberDomain::Integer) => {
								Some(Ordering::Equal)
							}
							(NumberDomain::Rational, NumberDomain::Integer) => {
								Some(Ordering::Greater)
							}
							_ => None
						}
					}
					Ordering::Greater => {
						match (self.domain, other.domain) {
							(NumberDomain::Rational, _) | (NumberDomain::Integer, NumberDomain::Integer) => {
								Some(Ordering::Greater)
							}
							_ => None
						}
					}
				}
			}
			Ordering::Greater => {
				match self.max.cmp(&other.max) {
					Ordering::Less | Ordering::Equal => {
						match (self.domain, other.domain) {
							(_, NumberDomain::Rational) | (NumberDomain::Integer, NumberDomain::Integer) => {
								Some(Ordering::Less)
							}
							_ => None
						}
					}
					Ordering::Greater => {
						None
					}
				}
			}
		}
	}

	pub fn contains(&self, _value: &value::Number) -> bool {
		true
	}

	pub fn domain(&self) -> NumberDomain {
		self.domain
	}

	pub fn set_domain(&mut self, d: NumberDomain) {
		self.domain = d;
		self.min.normalize(d);
		self.max.normalize(d);
	}

	pub fn min(&self) -> &MinBound {
		&self.min
	}

	pub fn set_min(&mut self, bound: MinBound) {
		self.min = bound;
		self.min.normalize(self.domain);
	}

	pub fn max(&self) -> &MaxBound {
		&self.max
	}

	pub fn set_max(&mut self, bound: MaxBound) {
		self.max = bound;
		self.max.normalize(self.domain);
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextString {
	automaton: iregex::automata::NFA<u32>,
	singleton: Ignored<OnceLock<Option<String>>>
}

impl Default for TextString {
	fn default() -> Self {
		let mut any_char = RangeSet::new();
		any_char.insert(AnyRange::new(Bound::Included('\0'), Bound::Included('\u{D7FF}')));
		any_char.insert(AnyRange::new(Bound::Included('\u{E000}'), Bound::Included('\u{10FFFF}')));

		let mut automaton = iregex::automata::NFA::new();
		automaton.add_initial_state(0);
		automaton.add(0, Some(any_char), 0);
		automaton.add_final_state(0);

		Self {
			automaton,
			singleton: Ignored(OnceLock::new())
		}
	}
}

impl TextString {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn singleton(value: String) -> Self {
		let automaton = iregex::automata::NFA::singleton(value.chars(), |i| match i {
			Some(i) => i as u32 + 1,
			None => 0
		});

		let singleton = OnceLock::new();
		let _ = singleton.set(Some(value));

		Self {
			automaton,
			singleton: Ignored(singleton)
		}
	}

	pub fn is_singleton(&self) -> bool {
		self.automaton.is_singleton()
	}

	pub fn as_singleton(&self) -> Option<&str> {
		self.singleton.get_or_init(|| {
			self.automaton.to_singleton().map(|chars| chars.into_iter().collect())
		}).as_deref()
	}

	pub fn automaton(&self) -> &iregex::automata::NFA<u32> {
		&self.automaton
	}

	pub fn set_automaton(&mut self, aut: iregex::automata::NFA<u32>) {
		self.automaton = aut;
		self.singleton = Ignored(OnceLock::new())
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self.as_singleton(), other.as_singleton()) {
			(Some(a), Some(b)) => if a == b {
				Some(Ordering::Equal)
			} else {
				None
			},
			(None, Some(b)) => if self.automaton.contains(b.chars()) {
				Some(Ordering::Greater)
			} else {
				None
			}
			(Some(a), None) => if self.automaton.contains(a.chars()) {
				Some(Ordering::Less)
			} else {
				None
			}
			(None, None) => {
				if self.automaton == other.automaton {
					Some(Ordering::Equal)
				} else {
					None
				}
			}
		}
	}

	pub fn contains(&self, _value: &str) -> bool {
		true
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteString;

impl ByteString {
	pub fn subtype_cmp(&self, _other: &Self) -> Option<Ordering> {
		Some(Ordering::Equal)
	}

	pub fn contains(&self, _value: &[u8]) -> bool {
		true
	}
}