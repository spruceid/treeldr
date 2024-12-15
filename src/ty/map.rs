use core::fmt;
use std::{cmp::Ordering, collections::BTreeMap, hash::Hash, marker::PhantomData};

use serde::{Deserialize, Serialize};
use serde_seeded::{unseeded, unseeded_btreemap_key, DeserializeSeeded, SerializeSeeded};

use crate::{
	display::DisplaySignature,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	utils::{self, take_first},
	Attributes, Registry, Value,
};

use super::TypeRef;

/// Map type.
#[derive(
	Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>), bounds(T: Ord))
)]
pub struct Map<P, R, T = TypeRef<P>> {
	/// Fields definitions.
	///
	/// # Soundness
	///
	/// It is a logical error for two keys to have a non-null intersection.
	#[seeded(with(unseeded_btreemap_key))]
	pub fields: BTreeMap<T, Field<P, R, T>>,
}

impl<P, R, T> Map<P, R, T> {
	pub fn new(fields: BTreeMap<T, Field<P, R, T>>) -> Self {
		Self { fields }
	}
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

impl<P, R, T> VisitResources<R> for Map<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		for field in self.fields.values() {
			field.visit_resources_ref_mut(f)
		}
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
				field.r#type.display_signature_with(context.inner())
			)?;
		}

		write!(f, "}}")
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>), bounds(T: Ord))
)]
pub struct Field<P, R, T = TypeRef<P>> {
	pub attributes: Attributes<R>,

	#[seeded(with(unseeded))]
	pub r#type: T,

	pub required: bool,

	#[seeded(skip)]
	p: PhantomData<P>,
}

impl<P, R, T> Field<P, R, T> {
	pub fn new(attributes: Attributes<R>, r#type: T, required: bool) -> Self {
		Self {
			attributes,
			r#type,
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
		(!other.required || self.required) && self.r#type.is_subtype_of(registry, other.r#type)
	}
}

impl<P, R, T> VisitResources<R> for Field<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.attributes.visit_resources_ref_mut(f);
	}
}
