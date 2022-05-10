use crate::{error, Error};
use locspan::Location;
use treeldr::{Caused, Id, MaybeSet, Name, WithCauses};

pub trait IntersectedWith<F>: Sized {
	fn intersected_with(
		self,
		id: Id,
		other: &Self,
		name: MaybeSet<Name, F>,
		cause: Option<&Location<F>>,
	) -> Result<Self, Error<F>>;
}

impl<F: Clone + Ord> IntersectedWith<F> for treeldr::layout::Enum<F> {
	fn intersected_with(
		self,
		id: Id,
		other: &Self,
		name: MaybeSet<Name, F>,
		cause: Option<&Location<F>>,
	) -> Result<Self, Error<F>> {
		let this = self.into_parts();
		let mut variants = Vec::new();

		let mut j = 0;
		'next_variant: for variant in this.variants {
			let (variant, variant_causes) = variant.into_parts();
			for (k, other_variant) in other.variants()[j..].iter().enumerate() {
				if variant.name() == other_variant.name() {
					if variant.layout() != other_variant.layout() {
						return Err(Caused::new(
							error::LayoutIntersectionFailed { id }.into(),
							cause.cloned(),
						));
					}

					let variant = variant.into_parts();

					let doc = if variant.doc.is_empty() || other_variant.documentation().is_empty()
					{
						variant.doc
					} else {
						other_variant.documentation().clone()
					};

					variants.push(WithCauses::new(
						treeldr::layout::Variant::new(
							variant.name,
							variant.layout.clone(),
							variant
								.label
								.clone()
								.or_else(|| other_variant.label().map(|l| l.to_string())),
							doc,
						),
						variant_causes.with(other_variant.causes().iter().cloned()),
					));

					j += k;
					continue 'next_variant;
				}
			}
		}

		Ok(Self::new(name.unwrap().unwrap_or(this.name), variants))
	}
}
