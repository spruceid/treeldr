use core::fmt;
use std::{cmp::Ordering, collections::BTreeMap, hash::Hash, marker::PhantomData};

use crate::{
	display::DisplaySignature,
	utils::{self, take_first},
	Attributes, Registry, Value,
};

use super::TypeRef;

/// Map type.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Map<P, R, T = TypeRef<P>> {
	/// Fields definitions.
	///
	/// # Soundness
	///
	/// It is a logical error for two keys to have a non-null intersection.
	pub fields: BTreeMap<T, Field<P, R, T>>,
}

impl<P: Copy + Ord, R: Ord> Map<P, R> {
	pub fn contains(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		map: &BTreeMap<Value<R>, Value<R>>,
	) -> bool {
		let mut fields: Vec<_> = self.fields.iter().collect();

		for key in map.keys() {
			if take_first(&mut fields, |(ty, _)| ty.contains(registry, key)).is_none()
				&& !self.fields.keys().any(|ty| ty.contains(registry, key))
			{
				return false; // unexpected field
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

impl<P: Copy + Ord + Hash, R: Ord> Map<P, R> {
	pub fn subtype_cmp(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other: &Self,
	) -> Option<Ordering> {
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

	pub fn is_subtype_of(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other: &Self,
	) -> bool {
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
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for Map<P, R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
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
pub struct Field<P, R, T = TypeRef<P>> {
	pub attributes: Attributes<R>,
	pub type_: T,
	pub required: bool,
	p: PhantomData<P>,
}

impl<P, R, T> Field<P, R, T> {
	pub fn new(attributes: Attributes<R>, type_: T, required: bool) -> Self {
		Self {
			attributes,
			type_,
			required,
			p: PhantomData,
		}
	}
}

impl<P: Copy + Ord + Hash, R: Ord> Field<P, R> {
	pub fn is_subfield_of(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other: &Self,
	) -> bool {
		(!other.required || self.required) && self.type_.is_subtype_of(registry, other.type_)
	}
}
