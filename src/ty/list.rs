use core::fmt;
use std::cmp::Ordering;

use crate::{display::DisplaySignature, Registry, Value};

use super::{refine_cmp, TypeRef};

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
