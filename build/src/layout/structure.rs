use crate::{error, Error};
use locspan::Location;
use treeldr::{vocab, Caused, Id, MaybeSet};
use vocab::Name;

pub trait IntersectedWith<F>: Sized {
	fn intersected_with(
		self,
		id: Id,
		other: &Self,
		name: MaybeSet<Name, F>,
		cause: Option<&Location<F>>,
	) -> Result<Self, Error<F>>;
}

impl<F: Ord + Clone> IntersectedWith<F> for treeldr::layout::Struct<F> {
	fn intersected_with(
		self,
		id: Id,
		other: &Self,
		name: MaybeSet<Name, F>,
		cause: Option<&Location<F>>,
	) -> Result<Self, Error<F>>
	where
		F: Clone + Ord,
	{
		let mut fields = Vec::new();

		let mut j = 0;

		let s = self.into_parts();

		for field in s.fields {
			let field = field.into_parts();

			for (k, other_field) in other.fields()[j..].iter().enumerate() {
				if field.name.inner() == other_field.name() {
					if *field.prop != other_field.property() {
						return Err(Caused::new(
							error::LayoutIntersectionFailed { id }.into(),
							cause.cloned(),
						));
					}

					if *field.layout != other_field.layout() {
						return Err(Caused::new(
							error::LayoutIntersectionFailed { id }.into(),
							cause.cloned(),
						));
					}

					let required = if *field.required || !other_field.is_required() {
						field.required.clone()
					} else {
						other_field.is_required_with_causes().clone()
					};

					let doc = if field.doc.is_empty() || other_field.documentation().is_empty() {
						field.doc.clone()
					} else {
						other_field.documentation().clone()
					};

					fields.push(treeldr::layout::Field::new(
						field.prop.clone(),
						field.name.clone(),
						field
							.label
							.clone()
							.or_else(|| other_field.label().map(|l| l.to_string())),
						field.layout.clone(),
						required,
						doc,
					));

					j += k;
				}
			}
		}

		Ok(Self::new(name.unwrap().unwrap_or(s.name), fields))
	}
}
