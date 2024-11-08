use crate::{
	display::DisplaySignature,
	utils::{self, take_first, Ignored},
	value::{
		self, NUMBER_I128_MAX, NUMBER_I128_MIN, NUMBER_I16_MAX, NUMBER_I16_MIN, NUMBER_I32_MAX,
		NUMBER_I32_MIN, NUMBER_I64_MAX, NUMBER_I64_MIN, NUMBER_I8_MAX, NUMBER_I8_MIN,
		NUMBER_U128_MAX, NUMBER_U16_MAX, NUMBER_U32_MAX, NUMBER_U64_MAX, NUMBER_U8_MAX,
		NUMBER_ZERO,
	},
	Attributes, Literal, Registry, Value,
};
use core::fmt;
use educe::Educe;
use iregex::automata::{AnyRange, Automaton, RangeSet};
use std::{
	cmp::Ordering,
	collections::{BTreeMap, BTreeSet},
	fmt::{Debug, Write},
	hash::Hash,
	ops::Bound,
	sync::OnceLock,
};

mod r#ref;
pub use r#ref::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltInType {
	Any,
	Resource,
	Unit,
	Boolean,
	Number,
	ByteString,
	TextString,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralType {
	Unit,
	Boolean(Boolean),
	Number(Number),
	ByteString(ByteString),
	TextString(TextString),
}

impl LiteralType {
	pub fn boolean() -> Self {
		Self::Boolean(Boolean::default())
	}

	pub fn number() -> Self {
		Self::Number(Number::default())
	}

	pub fn byte_string() -> Self {
		Self::ByteString(ByteString::default())
	}

	pub fn text_string() -> Self {
		Self::TextString(TextString::default())
	}

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
			_ => None,
		}
	}
}

impl<R> DisplaySignature<R> for LiteralType {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self {
			Self::Unit => write!(f, "null"),
			Self::Boolean(ty) => ty.fmt_signature(context, f),
			Self::Number(ty) => ty.fmt_signature(context, f),
			Self::ByteString(ty) => ty.fmt_signature(context, f),
			Self::TextString(ty) => ty.fmt_signature(context, f),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Type<R, T = TypeRef> {
	pub attributes: Attributes<R>,
	pub desc: TypeDesc<R, T>,
}

impl<R, T> Type<R, T> {
	pub fn new(attributes: Attributes<R>, desc: TypeDesc<R, T>) -> Self {
		Self { attributes, desc }
	}

	pub fn resource() -> Self {
		Self {
			attributes: Attributes::new(),
			desc: TypeDesc::resource(),
		}
	}

	pub fn is_resource(&self) -> bool {
		self.desc.is_resource()
	}

	pub fn as_list(&self) -> Option<&List<T>> {
		self.desc.as_list()
	}

	pub fn as_struct(&self) -> Option<&Struct<T>> {
		self.desc.as_struct()
	}

	pub fn visit_type_components(&self, f: impl FnMut(&T, bool)) {
		self.desc.visit_type_components(f);
	}
}

impl<R, T> From<TypeDesc<R, T>> for Type<R, T> {
	fn from(desc: TypeDesc<R, T>) -> Self {
		Self {
			attributes: Attributes::new(),
			desc,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeDesc<R, T = TypeRef> {
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

impl<R, T> TypeDesc<R, T> {
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

	pub fn as_struct(&self) -> Option<&Struct<T>> {
		match self {
			Self::Struct(m) => Some(m),
			_ => None,
		}
	}

	pub fn visit_type_components(&self, mut f: impl FnMut(&T, bool)) {
		match self {
			Self::Any | Self::Resource(_) | Self::Literal(_) => (),
			Self::List(ty) => {
				for c_ref in &ty.prefix {
					f(c_ref, false);
				}

				if let Some(c_ref) = &ty.rest {
					f(c_ref, false);
				}
			}
			Self::Struct(ty) => {
				for (key, field) in &ty.fields {
					f(key, true);
					f(&field.type_, false);
				}
			}
			Self::Enum(ty) => {
				for c_ref in ty.variants.values() {
					f(c_ref, false);
				}
			}
		}
	}
}

pub trait TypeUnifier<T, U> {
	type Target;

	fn left_key_type(&mut self, t: &T) -> TypeRef;

	fn right_key_type(&mut self, u: &U) -> TypeRef;

	fn unify(&mut self, t: &T, u: &U) -> Self::Target;
}

impl<R: Clone + Ord, T> Type<R, T> {
	pub fn unify_with<U, V>(
		&self,
		registry: &Registry<R>,
		unifier: &mut impl TypeUnifier<T, U, Target = V>,
		other: &Type<R, U>,
	) -> Result<Type<R, V>, TypeUnificationError>
	where
		V: Ord,
	{
		let attributes = if self.attributes.is_empty() {
			other.attributes.clone()
		} else if other.attributes.is_empty() || self.attributes == other.attributes {
			self.attributes.clone()
		} else {
			return Err(TypeUnificationError);
		};

		let desc = self.desc.unify_with(registry, unifier, &other.desc)?;

		Ok(Type::new(attributes, desc))
	}
}

impl<R: Clone + Ord, T> TypeDesc<R, T> {
	pub fn unify_with<U, V>(
		&self,
		registry: &Registry<R>,
		unifier: &mut impl TypeUnifier<T, U, Target = V>,
		other: &TypeDesc<R, U>,
	) -> Result<TypeDesc<R, V>, TypeUnificationError>
	where
		V: Ord,
	{
		match (self, other) {
			(Self::Any, TypeDesc::Any) => Ok(TypeDesc::Any),
			(Self::Resource(a), TypeDesc::Resource(b)) => {
				if a == b {
					Ok(TypeDesc::Resource(a.clone()))
				} else {
					Err(TypeUnificationError)
				}
			}
			(Self::Literal(a), TypeDesc::Literal(b)) => match a.subtype_cmp(b) {
				Some(Ordering::Equal) => Ok(TypeDesc::Literal(a.clone())),
				_ => Err(TypeUnificationError),
			},
			(Self::List(a), TypeDesc::List(b)) => Ok(TypeDesc::List(List {
				prefix: if a.prefix.len() == b.prefix.len() {
					a.prefix
						.iter()
						.zip(&b.prefix)
						.map(|(a, b)| unifier.unify(a, b))
						.collect()
				} else {
					return Err(TypeUnificationError);
				},
				rest: match (&a.rest, &b.rest) {
					(Some(a), Some(b)) => Some(unifier.unify(a, b)),
					(None, None) => None,
					_ => return Err(TypeUnificationError),
				},
			})),
			(Self::Struct(a), TypeDesc::Struct(b)) => {
				if a.fields.len() == b.fields.len() {
					let mut fields = BTreeMap::new();

					for (a_key, a_field) in &a.fields {
						let a_key_type = unifier.left_key_type(a_key);
						let b_entry = b.fields.iter().find(|(k, _)| {
							a_key_type
								.subtype_cmp(registry, unifier.right_key_type(k))
								.is_some_and(Ordering::is_eq)
						});

						match b_entry {
							Some((b_key, b_field)) => {
								if a_field.required == b_field.required {
									let key = unifier.unify(a_key, b_key);
									fields.insert(
										key,
										Field {
											type_: unifier.unify(&a_field.type_, &b_field.type_),
											required: a_field.required,
										},
									);
								} else {
									return Err(TypeUnificationError);
								}
							}
							None => return Err(TypeUnificationError),
						}
					}

					Ok(TypeDesc::Struct(Struct { fields }))
				} else {
					Err(TypeUnificationError)
				}
			}
			(Self::Enum(a), TypeDesc::Enum(b)) => {
				if a.variants.len() == b.variants.len() {
					let mut variants = BTreeMap::new();

					for (key, a_type) in &a.variants {
						match b.variants.get(key) {
							Some(b_type) => {
								variants.insert(key.clone(), unifier.unify(a_type, b_type));
							}
							None => return Err(TypeUnificationError),
						}
					}

					Ok(TypeDesc::Enum(Enum { variants }))
				} else {
					Err(TypeUnificationError)
				}
			}
			_ => Err(TypeUnificationError),
		}
	}
}

#[derive(Debug, thiserror::Error)]
#[error("type unification failed")]
pub struct TypeUnificationError;

impl<R: Ord> Type<R> {
	pub fn subtype_cmp(&self, registry: &Registry<R>, other: &Self) -> Option<Ordering> {
		self.desc.subtype_cmp(registry, &other.desc)
	}

	pub fn contains(&self, registry: &Registry<R>, value: &Value<R>) -> bool {
		self.desc.contains(registry, value)
	}
}

impl<R: Ord> TypeDesc<R> {
	pub fn subtype_cmp(&self, registry: &Registry<R>, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(Self::Any, Self::Any) => Some(Ordering::Equal),
			(Self::Any, _) => Some(Ordering::Greater),
			(_, Self::Any) => Some(Ordering::Less),
			(Self::Resource(a), Self::Resource(b)) => a.subtype_cmp(b),
			(Self::Literal(a), Self::Literal(b)) => a.subtype_cmp(b),
			(Self::List(a), Self::List(b)) => a.subtype_cmp(registry, b),
			(Self::Struct(a), Self::Struct(b)) => a.subtype_cmp(registry, b),
			(Self::Enum(a), b) => a.subtype_cmp(registry, b),
			(a, Self::Enum(b)) => b.subtype_cmp(registry, a).map(Ordering::reverse),
			_ => None,
		}
	}

	pub fn contains(&self, registry: &Registry<R>, value: &Value<R>) -> bool {
		match (self, value) {
			(Self::Any, _) => true,
			(Self::Resource(ty), Value::Resource(r)) => ty.contains(r),
			(Self::Literal(ty), Value::Literal(l)) => ty.contains(l),
			(Self::List(ty), Value::List(list)) => ty.contains(registry, list),
			(Self::Struct(ty), Value::Map(map)) => ty.contains(registry, map),
			(Self::Enum(ty), value) => ty.contains(registry, value),
			_ => false,
		}
	}
}

impl<R: fmt::Display> DisplaySignature<R> for Type<R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		self.desc.fmt_signature(context, f)
	}
}

impl<R: fmt::Display> DisplaySignature<R> for TypeDesc<R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self {
			Self::Any => write!(f, "any"),
			Self::Resource(ty) => ty.fmt_signature(context, f),
			Self::Literal(ty) => ty.fmt_signature(context, f),
			Self::List(ty) => ty.fmt_signature(context, f),
			Self::Struct(ty) => ty.fmt_signature(context, f),
			Self::Enum(ty) => ty.fmt_signature(context, f),
		}
	}
}

/// Resource type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Educe)]
#[educe(Default)]
pub struct Resource<R> {
	set: Option<BTreeSet<R>>,
}

impl<R: Ord> Resource<R> {
	pub fn singleton(r: R) -> Self {
		let mut set = BTreeSet::new();
		set.insert(r);

		Self { set: Some(set) }
	}

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
			(None, None) => Some(Ordering::Equal),
		}
	}

	pub fn contains(&self, resource: &R) -> bool {
		match &self.set {
			Some(set) => set.contains(resource),
			None => true,
		}
	}
}

impl<R: fmt::Display> DisplaySignature<R> for Resource<R> {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match &self.set {
			Some(set) => {
				write!(f, "(any_of")?;

				for r in set {
					write!(f, " {r}")?;
				}

				write!(f, ")")
			}
			None => {
				write!(f, "resource")
			}
		}
	}
}

/// Struct type.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Struct<T = TypeRef> {
	/// Fields definitions.
	///
	/// # Soundness
	///
	/// It is a logical error for two keys to have a non-null intersection.
	pub fields: BTreeMap<T, Field<T>>,
}

impl Struct<TypeRef> {
	pub fn subtype_cmp<R: Ord>(&self, registry: &Registry<R>, other: &Self) -> Option<Ordering> {
		if self.is_subtype_of(registry, other) {
			if other.is_subtype_of(registry, self) {
				Some(Ordering::Equal)
			} else {
				Some(Ordering::Less)
			}
		} else if other.is_subtype_of(registry, self) {
			Some(Ordering::Greater)
		} else {
			None
		}
	}

	pub fn is_subtype_of<R: Ord>(&self, registry: &Registry<R>, other: &Self) -> bool {
		let mut fields: Vec<_> = other.fields.iter().collect();

		for (key, field) in &self.fields {
			let other_field = utils::take_first(&mut fields, |(&other_key, _)| {
				key.is_subtype_of(registry, other_key)
			})
			.or_else(|| {
				other
					.fields
					.iter()
					.find(|(&other_key, _)| key.is_subtype_of(registry, other_key))
			});

			match other_field {
				Some((_, other_field)) => {
					if !field.is_subfield_of(registry, other_field) {
						return false;
					}
				}
				None => return false,
			}
		}

		fields.into_iter().all(|(_, field)| !field.required)
	}

	pub fn contains<R: Ord>(
		&self,
		registry: &Registry<R>,
		map: &BTreeMap<Value<R>, Value<R>>,
	) -> bool {
		let mut fields: Vec<_> = self.fields.iter().collect();

		for (key, _) in map {
			if take_first(&mut fields, |(ty, _)| ty.contains(registry, key)).is_none() {
				if !self.fields.keys().any(|ty| ty.contains(registry, key)) {
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

impl<R: fmt::Display> DisplaySignature<R> for Struct {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		write!(f, "{{")?;

		for (i, (key, field)) in self.fields.iter().enumerate() {
			if i > 0 {
				write!(f, ", ")?;
			}

			key.fmt_signature(&context.inner(), f)?;

			if !field.required {
				write!(f, "?")?;
			}

			write!(
				f,
				": {}",
				field.type_.display_signature_with(context.inner())
			)?;
		}

		write!(f, "}}")
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Field<T> {
	pub type_: T,
	pub required: bool,
}

impl Field<TypeRef> {
	pub fn is_subfield_of<R: Ord>(&self, registry: &Registry<R>, other: &Self) -> bool {
		(!other.required || self.required) && self.type_.is_subtype_of(registry, other.type_)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enum<T = TypeRef> {
	pub variants: BTreeMap<String, T>,
}

impl Enum<TypeRef> {
	pub fn subtype_cmp<R: Ord>(
		&self,
		registry: &Registry<R>,
		other: &TypeDesc<R>,
	) -> Option<Ordering> {
		match other {
			TypeDesc::Enum(other) => {
				let mut cmp = Ordering::Equal;

				for &a in self.variants.values() {
					for &b in other.variants.values() {
						cmp = refine_cmp(cmp, a.subtype_cmp(registry, b))?
					}
				}

				Some(cmp)
			}
			other => {
				let mut cmp = None;
				let mut leq = true;

				for (_, &ty_ref) in &self.variants {
					let ty: &Type<R> = registry.get_type(ty_ref)?;
					match (cmp, ty.desc.subtype_cmp(registry, other)) {
						(
							None | Some(Ordering::Greater | Ordering::Equal),
							Some(Ordering::Greater),
						) => {
							leq = false;
							cmp = Some(Ordering::Greater)
						}
						(None | Some(Ordering::Equal), Some(Ordering::Equal)) => {
							cmp = Some(Ordering::Equal)
						}
						(_, None) => leq = false,
						_ => (),
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

	pub fn contains<R: Ord>(&self, registry: &Registry<R>, value: &Value<R>) -> bool {
		self.variants
			.values()
			.any(|ty| ty.contains(registry, value))
	}
}

impl<R: fmt::Display> DisplaySignature<R> for Enum {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		write!(f, "enum {{")?;

		for (name, variant) in &self.variants {
			write!(
				f,
				"{name}: {}",
				variant.display_signature_with(context.inner())
			)?;
		}

		write!(f, "}}")
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct List<T = TypeRef> {
	pub prefix: Vec<T>,
	pub rest: Option<T>,
}

impl<T> List<T> {
	pub fn from_prefix(prefix: Vec<T>) -> Self {
		Self { prefix, rest: None }
	}

	pub fn uniform(ty: T) -> Self {
		Self {
			prefix: Vec::new(),
			rest: Some(ty),
		}
	}
}

impl List<TypeRef> {
	pub fn item_type(&self, i: usize) -> Option<TypeRef> {
		if i < self.prefix.len() {
			Some(self.prefix[i])
		} else {
			self.rest
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

	pub fn as_uniform(&self) -> Option<TypeRef> {
		if self.prefix.is_empty() {
			self.rest
		} else {
			None
		}
	}

	pub fn subtype_cmp<R: Ord>(&self, registry: &Registry<R>, other: &Self) -> Option<Ordering> {
		let mut a_prefix = self.prefix.iter().copied();
		let mut b_prefix = other.prefix.iter().copied();

		let mut cmp = Ordering::Equal;

		loop {
			cmp = match (a_prefix.next(), b_prefix.next()) {
				(Some(a), Some(b)) => refine_cmp(cmp, a.subtype_cmp(registry, b)),
				(Some(a), None) => refine_cmp(cmp, a.subtype_cmp(registry, other.rest?)),
				(None, Some(b)) => refine_cmp(cmp, self.rest.as_ref()?.subtype_cmp(registry, b)),
				(None, None) => break,
			}?
		}

		let rest_cmp = match (self.rest, other.rest) {
			(Some(a), Some(b)) => a.subtype_cmp(registry, b),
			(Some(_), None) => Some(Ordering::Greater),
			(None, Some(_)) => Some(Ordering::Less),
			(None, None) => Some(Ordering::Equal),
		};

		refine_cmp(cmp, rest_cmp)
	}

	pub fn contains<R: Ord>(&self, registry: &Registry<R>, value: &[Value<R>]) -> bool {
		if value.len() < self.min_len() {
			return false;
		}

		value.iter().enumerate().all(|(i, item)| {
			self.item_type(i)
				.is_some_and(|ty| ty.contains(registry, item))
		})
	}
}

impl<R: fmt::Display> DisplaySignature<R> for List {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		write!(f, "[")?;

		for (i, &p) in self.prefix.iter().enumerate() {
			if i > 0 {
				write!(f, ", ")?;
			}

			p.fmt_signature(&context.inner(), f)?;
		}

		if let Some(t) = self.rest {
			if !self.prefix.is_empty() {
				write!(f, ", ")?;
			}

			write!(f, "... {}", t.display_signature_with(context.inner()))?;
		}

		write!(f, "]")
	}
}

fn refine_cmp(cmp: Ordering, refine_with: Option<Ordering>) -> Option<Ordering> {
	match (cmp, refine_with?) {
		(Ordering::Equal, cmp) => Some(cmp),
		(Ordering::Greater, Ordering::Greater | Ordering::Equal) => Some(Ordering::Greater),
		(Ordering::Less, Ordering::Less | Ordering::Equal) => Some(Ordering::Less),
		_ => None,
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Boolean {
	const_: Option<bool>,
}

impl Boolean {
	pub fn singleton(b: bool) -> Self {
		Self { const_: Some(b) }
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self.const_, other.const_) {
			(Some(a), Some(b)) => {
				if a == b {
					Some(Ordering::Equal)
				} else {
					None
				}
			}
			(None, Some(_)) => Some(Ordering::Greater),
			(Some(_), None) => Some(Ordering::Less),
			(None, None) => Some(Ordering::Equal),
		}
	}

	pub fn contains(&self, value: bool) -> bool {
		match self.const_ {
			Some(b) => b == value,
			None => true,
		}
	}
}

impl<R> DisplaySignature<R> for Boolean {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self.const_ {
			Some(true) => write!(f, "true"),
			Some(false) => write!(f, "false"),
			None => write!(f, "bool"),
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumberDomain {
	/// Rational numbers (ℚ).
	#[default]
	Rational,

	/// Integer numbers (ℤ).
	Integer,
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
	Included(value::Number),
}

impl MinBound {
	pub fn is_unbounded(&self) -> bool {
		matches!(self, Self::Unbounded)
	}

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
	pub fn is_unbounded(&self) -> bool {
		matches!(self, Self::Unbounded)
	}

	pub fn as_included(&self) -> Option<&value::Number> {
		match self {
			Self::Included(m) => Some(m),
			_ => None,
		}
	}

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
	max: MaxBound,
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
			max: MaxBound::Included(value),
		}
	}

	pub fn as_singleton(&self) -> Option<&value::Number> {
		match (&self.min, &self.max) {
			(MinBound::Included(a), MaxBound::Included(b)) if a == b => Some(a),
			_ => None,
		}
	}

	pub fn subtype_cmp(&self, other: &Self) -> Option<Ordering> {
		match self.min.cmp(&other.min) {
			Ordering::Less => match self.max.cmp(&other.max) {
				Ordering::Less => None,
				Ordering::Greater | Ordering::Equal => match (self.domain, other.domain) {
					(NumberDomain::Rational, _)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Greater),
					_ => None,
				},
			},
			Ordering::Equal => match self.max.cmp(&other.max) {
				Ordering::Less => match (self.domain, other.domain) {
					(_, NumberDomain::Rational)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Less),
					_ => None,
				},
				Ordering::Equal => match (self.domain, other.domain) {
					(NumberDomain::Rational, NumberDomain::Rational)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Equal),
					(NumberDomain::Rational, NumberDomain::Integer) => Some(Ordering::Greater),
					_ => None,
				},
				Ordering::Greater => match (self.domain, other.domain) {
					(NumberDomain::Rational, _)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Greater),
					_ => None,
				},
			},
			Ordering::Greater => match self.max.cmp(&other.max) {
				Ordering::Less | Ordering::Equal => match (self.domain, other.domain) {
					(_, NumberDomain::Rational)
					| (NumberDomain::Integer, NumberDomain::Integer) => Some(Ordering::Less),
					_ => None,
				},
				Ordering::Greater => None,
			},
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

	fn as_native_number(&self) -> Option<NativeNumber> {
		match self.domain {
			NumberDomain::Integer => {
				// int
				match &self.min {
					MinBound::Unbounded => Some(NativeNumber::Int),
					MinBound::Included(n) => {
						if *n == *NUMBER_ZERO {
							match &self.max {
								MaxBound::Unbounded => return Some(NativeNumber::Uint),
								MaxBound::Included(m) => {
									if *m == *NUMBER_U8_MAX {
										return Some(NativeNumber::U8);
									}

									if *m == *NUMBER_U16_MAX {
										return Some(NativeNumber::U16);
									}

									if *m == *NUMBER_U32_MAX {
										return Some(NativeNumber::U32);
									}

									if *m == *NUMBER_U64_MAX {
										return Some(NativeNumber::U64);
									}

									if *m == *NUMBER_U128_MAX {
										return Some(NativeNumber::U128);
									}
								}
								MaxBound::Excluded(_) => panic!("non normalized bound"),
							}
						}

						if *n == *NUMBER_I8_MIN
							&& self.max.as_included().is_some_and(|m| *m == *NUMBER_I8_MAX)
						{
							return Some(NativeNumber::I8);
						}

						if *n == *NUMBER_I16_MIN
							&& self
								.max
								.as_included()
								.is_some_and(|m| *m == *NUMBER_I16_MAX)
						{
							return Some(NativeNumber::I16);
						}

						if *n == *NUMBER_I32_MIN
							&& self
								.max
								.as_included()
								.is_some_and(|m| *m == *NUMBER_I32_MAX)
						{
							return Some(NativeNumber::I32);
						}

						if *n == *NUMBER_I64_MIN
							&& self
								.max
								.as_included()
								.is_some_and(|m| *m == *NUMBER_I64_MAX)
						{
							return Some(NativeNumber::I64);
						}

						if *n == *NUMBER_I128_MIN
							&& self
								.max
								.as_included()
								.is_some_and(|m| *m == *NUMBER_I128_MAX)
						{
							return Some(NativeNumber::I128);
						}

						None
					}
					MinBound::Excluded(_) => panic!("non normalized bound"),
				}
			}
			NumberDomain::Rational => {
				if self.min.is_unbounded() && self.max.is_unbounded() {
					Some(NativeNumber::Rational)
				} else {
					None
				}
			}
		}
	}
}

impl<R> DisplaySignature<R> for Number {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self.as_singleton() {
			Some(s) => fmt::Display::fmt(s, f),
			None => match self.as_native_number() {
				Some(n) => match n {
					NativeNumber::Uint => write!(f, "uint"),
					NativeNumber::U8 => write!(f, "u8"),
					NativeNumber::U16 => write!(f, "u16"),
					NativeNumber::U32 => write!(f, "u32"),
					NativeNumber::U64 => write!(f, "u64"),
					NativeNumber::U128 => write!(f, "u128"),
					NativeNumber::Int => write!(f, "int"),
					NativeNumber::I8 => write!(f, "i8"),
					NativeNumber::I16 => write!(f, "i16"),
					NativeNumber::I32 => write!(f, "i32"),
					NativeNumber::I64 => write!(f, "i64"),
					NativeNumber::I128 => write!(f, "i128"),
					NativeNumber::Rational => write!(f, "rational"),
				},
				None => {
					todo!()
				}
			},
		}
	}
}

pub enum NativeNumber {
	Uint,
	U8,
	U16,
	U32,
	U64,
	U128,
	Int,
	I8,
	I16,
	I32,
	I64,
	I128,
	Rational,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TextStringPrecomputedData {
	is_any: bool,
	regex_string: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextString {
	automaton: iregex::automata::NFA<u32>,
	singleton: Ignored<OnceLock<Option<String>>>,
	precomputed_data: TextStringPrecomputedData,
}

impl Default for TextString {
	fn default() -> Self {
		let mut any_char = RangeSet::new();
		any_char.insert(AnyRange::new(
			Bound::Included('\0'),
			Bound::Included('\u{D7FF}'),
		));
		any_char.insert(AnyRange::new(
			Bound::Included('\u{E000}'),
			Bound::Included('\u{10FFFF}'),
		));

		let mut automaton = iregex::automata::NFA::new();
		automaton.add_initial_state(0);
		automaton.add(0, Some(any_char), 0);
		automaton.add_final_state(0);

		Self {
			automaton,
			singleton: Ignored(OnceLock::new()),
			precomputed_data: TextStringPrecomputedData {
				is_any: true,
				regex_string: ".*".to_owned(),
			},
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
			None => 0,
		});

		let mut regex_string = String::new();
		for c in value.chars() {
			match c {
				'(' => regex_string.push_str("\\("),
				')' => regex_string.push_str("\\)"),
				'[' => regex_string.push_str("\\["),
				']' => regex_string.push_str("\\]"),
				'\\' => regex_string.push_str("\\\\"),
				'.' => regex_string.push_str("\\."),
				'*' => regex_string.push_str("\\*"),
				'+' => regex_string.push_str("\\+"),
				'?' => regex_string.push_str("\\?"),
				'^' => regex_string.push_str("\\^"),
				c => regex_string.push(c),
			}
		}

		let singleton = OnceLock::new();
		let _ = singleton.set(Some(value));

		Self {
			automaton,
			singleton: Ignored(singleton),
			precomputed_data: TextStringPrecomputedData {
				is_any: false,
				regex_string,
			},
		}
	}

	pub fn is_singleton(&self) -> bool {
		self.automaton.is_singleton()
	}

	pub fn as_singleton(&self) -> Option<&str> {
		self.singleton
			.get_or_init(|| {
				self.automaton
					.to_singleton()
					.map(|chars| chars.into_iter().collect())
			})
			.as_deref()
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
			(Some(a), Some(b)) => {
				if a == b {
					Some(Ordering::Equal)
				} else {
					None
				}
			}
			(None, Some(b)) => {
				if self.automaton.contains(b.chars()) {
					Some(Ordering::Greater)
				} else {
					None
				}
			}
			(Some(a), None) => {
				if self.automaton.contains(a.chars()) {
					Some(Ordering::Less)
				} else {
					None
				}
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

	pub fn contains(&self, value: &str) -> bool {
		match self.as_singleton() {
			Some(singleton) => singleton == value,
			None => self.automaton.contains(value.chars()),
		}
	}
}

impl<R> DisplaySignature<R> for TextString {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		match self.as_singleton() {
			Some(s) => {
				f.write_char('"')?;

				for c in s.chars() {
					match c {
						'\t' => f.write_str("\\t")?,
						'\r' => f.write_str("\\r")?,
						'\n' => f.write_str("\\n")?,
						'\'' => f.write_str("\\'")?,
						'\"' => f.write_str("\\\"")?,
						'\\' => f.write_str("\\\\")?,
						c => f.write_char(c)?,
					}
				}

				f.write_char('"')
			}
			None => {
				if self.precomputed_data.is_any {
					write!(f, "string")
				} else {
					write!(f, "/{}/", self.precomputed_data.regex_string)
				}
			}
		}
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

impl<R> DisplaySignature<R> for ByteString {
	fn fmt_signature(
		&self,
		_context: &crate::display::DisplayContext<R>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		write!(f, "bytes")
	}
}
