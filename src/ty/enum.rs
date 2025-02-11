use core::fmt;
use std::{cmp::Ordering, collections::BTreeMap, hash::Hash, marker::PhantomData};

use serde::{Deserialize, Serialize};
use serde_seeded::{unseeded, DeserializeSeeded, SerializeSeeded};

use crate::{
	display::DisplaySignature,
	rdf::{ResourceBackwardIndex, ResourceForwardIndex, VisitResources},
	Attributes, Registry, Value,
};

use super::{refine_cmp, Type, TypeDesc, TypeRef};

#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeSeeded, DeserializeSeeded,
)]
#[seeded(
	ser(seed(ResourceForwardIndex<R>), override_bounds(P: Serialize, R: Eq + Hash, T: Serialize)),
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct Enum<P, R, T = TypeRef<P>> {
	pub variants: BTreeMap<String, Variant<P, R, T>>,
}

impl<P: Copy + Ord, R: Ord> Enum<P, R> {
	pub fn contains(
		&self,
		registry: &(impl ?Sized + Registry<PackageId = P, Resource = R>),
		value: &Value<R>,
	) -> bool {
		self.variants
			.values()
			.any(|variant| variant.r#type.contains(registry, value))
	}
}

impl<P: Copy + Ord + Hash, R: Ord> Enum<P, R> {
	pub fn subtype_cmp(
		&self,
		registry: &(impl ?Sized + Registry<PackageId = P, Resource = R>),
		other: &TypeDesc<P, R>,
	) -> Option<Ordering> {
		match other {
			TypeDesc::Enum(other) => {
				let mut cmp = Ordering::Equal;

				for a in self.variants.values() {
					for b in other.variants.values() {
						cmp = refine_cmp(cmp, a.r#type.subtype_cmp(registry, b.r#type))?
					}
				}

				Some(cmp)
			}
			other => {
				let mut cmp = None;
				let mut leq = true;

				for variant in self.variants.values() {
					let ty: &Type<P, R> = registry.get_type(variant.r#type)?;
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
}

impl<P, R, T> VisitResources<R> for Enum<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		for variant in self.variants.values() {
			variant.visit_resources_ref_mut(f)
		}
	}
}

impl<P: Copy + PartialEq, R: fmt::Display> DisplaySignature<P, R> for Enum<P, R> {
	fn fmt_signature(
		&self,
		context: &crate::display::DisplayContext<impl Registry<PackageId = P, Resource = R>>,
		f: &mut fmt::Formatter,
	) -> fmt::Result {
		write!(f, "enum {{")?;

		for (name, variant) in &self.variants {
			write!(
				f,
				"{name}: {}",
				variant.r#type.display_signature_with(context.inner())
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
	de(seed(ResourceBackwardIndex<R>), override_bounds(P: Deserialize<'de>, R: Clone + Ord, T: Deserialize<'de>))
)]
pub struct Variant<P, R, T = TypeRef<P>> {
	pub attributes: Attributes<R>,

	#[seeded(with(unseeded))]
	pub r#type: T,

	#[seeded(skip)]
	p: PhantomData<P>,
}

impl<P, R, T> Variant<P, R, T> {
	pub fn new(attributes: Attributes<R>, r#type: T) -> Self {
		Self {
			attributes,
			r#type,
			p: PhantomData,
		}
	}
}

impl<P, R, T> VisitResources<R> for Variant<P, R, T> {
	fn visit_resources_ref_mut(&self, f: &mut impl FnMut(&R)) {
		self.attributes.visit_resources_ref_mut(f)
	}
}
