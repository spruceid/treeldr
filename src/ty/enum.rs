use core::fmt;
use std::{cmp::Ordering, collections::BTreeMap};

use crate::{display::DisplaySignature, Attributes, Registry, Value};

use super::{refine_cmp, Type, TypeDesc, TypeRef};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enum<R, T = TypeRef> {
	pub variants: BTreeMap<String, Variant<R, T>>,
}

impl<R: Ord> Enum<R, TypeRef> {
	pub fn subtype_cmp(&self, registry: &Registry<R>, other: &TypeDesc<R>) -> Option<Ordering> {
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
					let ty: &Type<R> = registry.get_type(variant.type_)?;
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

	pub fn contains(&self, registry: &Registry<R>, value: &Value<R>) -> bool {
		self.variants
			.values()
			.any(|variant| variant.type_.contains(registry, value))
	}
}

impl<R: fmt::Display> DisplaySignature<R> for Enum<R> {
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
				variant.type_.display_signature_with(context.inner())
			)?;
		}

		write!(f, "}}")
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variant<R, T = TypeRef> {
	pub attributes: Attributes<R>,
	pub type_: T,
}
