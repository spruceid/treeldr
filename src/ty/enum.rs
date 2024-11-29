use core::fmt;
use std::{cmp::Ordering, collections::BTreeMap, hash::Hash, marker::PhantomData};

use crate::{display::DisplaySignature, Attributes, Registry, Value};

use super::{refine_cmp, Type, TypeDesc, TypeRef};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enum<P, R, T = TypeRef<P>> {
	pub variants: BTreeMap<String, Variant<P, R, T>>,
}

impl<P: Copy + Ord, R: Ord> Enum<P, R> {
	pub fn contains(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		value: &Value<R>,
	) -> bool {
		self.variants
			.values()
			.any(|variant| variant.type_.contains(registry, value))
	}
}

impl<P: Copy + Ord + Hash, R: Ord> Enum<P, R> {
	pub fn subtype_cmp(
		&self,
		registry: &impl Registry<PackageId = P, Resource = R>,
		other: &TypeDesc<P, R>,
	) -> Option<Ordering> {
		match other {
			TypeDesc::Enum(other) => {
				let mut cmp = Ordering::Equal;

				for a in self.variants.values() {
					for b in other.variants.values() {
						cmp = refine_cmp(cmp, a.type_.subtype_cmp(registry, b.type_))?
					}
				}

				Some(cmp)
			}
			other => {
				let mut cmp = None;
				let mut leq = true;

				for variant in self.variants.values() {
					let ty: &Type<P, R> = registry.get_type(variant.type_)?;
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
				variant.type_.display_signature_with(context.inner())
			)?;
		}

		write!(f, "}}")
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variant<P, R, T = TypeRef<P>> {
	pub attributes: Attributes<R>,
	pub type_: T,
	p: PhantomData<P>,
}

impl<P, R, T> Variant<P, R, T> {
	pub fn new(attributes: Attributes<R>, type_: T) -> Self {
		Self {
			attributes,
			type_,
			p: PhantomData,
		}
	}
}
