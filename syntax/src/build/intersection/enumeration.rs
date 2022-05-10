use super::{IntersectedLayout, IntersectedLayoutDescription};
use crate::build::{Descriptions, Error};
use treeldr::{Caused, Causes, Id, MaybeSet, Name, Vocabulary, WithCauses};
use treeldr_build::{Context, ObjectToId};

#[derive(Clone)]
pub struct IntersectedEnum<F> {
	variants: Vec<WithCauses<IntersectedVariant<F>, F>>,
}

impl<F: Clone + Ord> IntersectedEnum<F> {
	pub fn new(
		variants_id: Id,
		context: &Context<F, Descriptions>,
		causes: &Causes<F>,
	) -> Result<Self, Error<F>> {
		let mut variants = Vec::new();

		for variant_obj in context
			.require_list(variants_id, causes.preferred().cloned())?
			.iter(context)
		{
			let variant_obj = variant_obj?;
			let variant_id = variant_obj.as_id(variant_obj.causes().preferred())?;
			let variant = context
				.require_layout_variant(variant_id, variant_obj.causes().preferred().cloned())?;
			variants.push(WithCauses::new(
				IntersectedVariant {
					name: variant.name().cloned().into(),
					layout: IntersectedLayout::try_from_iter(
						variant.layout().with_causes().cloned(),
						context,
						variant_obj.causes().clone(),
					)?,
				},
				variant.causes().clone(),
			))
		}

		Ok(Self { variants })
	}

	pub fn intersected_with(
		mut self,
		mut other: IntersectedEnum<F>,
		context: &Context<F, Descriptions>,
	) -> Result<IntersectedLayoutDescription<F>, Error<F>> {
		let mut variants = std::mem::take(&mut self.variants);
		variants.reverse();
		other.variants.reverse();

		'next_variant: while !variants.is_empty() && !other.variants.is_empty() {
			if let Some(variant) = variants.pop() {
				let (variant, causes) = variant.into_parts();
				while let Some(other_variant) = other.variants.pop() {
					if variant.matches(&other_variant) {
						let (other_variant, other_causes) = other_variant.into_parts();
						if let Some(intersected_variant) =
							variant.intersected_with(other_variant, context)?
						{
							self.variants.push(WithCauses::new(
								intersected_variant,
								causes.with(other_causes),
							))
						}

						continue 'next_variant;
					} else {
						for after_variant in &variants {
							if after_variant.matches(&other_variant) {
								for j in 0..other.variants.len() {
									if variant.matches(&other.variants[j]) {
										panic!("unaligned layouts")
									}
								}

								other.variants.push(other_variant);
								continue 'next_variant;
							}
						}
					}
				}
			}
		}

		match self.variants.len() {
			0 => Ok(IntersectedLayoutDescription::Never),
			1 => {
				let variant = self.variants.into_iter().next().unwrap().into_inner();
				match variant.layout.id() {
					Some((id, _)) => Ok(IntersectedLayoutDescription::Alias(id)),
					None => Ok(variant.layout.into_description().into_inner()),
				}
			}
			_ => Ok(IntersectedLayoutDescription::Enum(self)),
		}
	}

	pub fn intersected_with_non_enum(
		mut self,
		other: IntersectedLayout<F>,
		context: &Context<F, Descriptions>,
	) -> Result<IntersectedLayoutDescription<F>, Error<F>> {
		let variants = std::mem::take(&mut self.variants);

		for variant in variants {
			let (mut variant, causes) = variant.into_parts();
			variant.layout = variant.layout.intersected_with(other.clone(), context)?;

			if variant.layout.has_id() {
				self.variants.push(WithCauses::new(variant, causes))
			}
		}

		match self.variants.len() {
			0 => Ok(IntersectedLayoutDescription::Never),
			1 => {
				let variant = self.variants.into_iter().next().unwrap().into_inner();
				match variant.layout.id() {
					Some((id, _)) => Ok(IntersectedLayoutDescription::Alias(id)),
					None => Ok(variant.layout.into_description().into_inner()),
				}
			}
			_ => Ok(IntersectedLayoutDescription::Enum(self)),
		}
	}

	pub fn into_standard_description(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
		match self.variants.len() {
			0 => Ok(treeldr_build::layout::Description::Never),
			1 => {
				let variant = self.variants.into_iter().next().unwrap().into_inner();
				match variant.layout.id() {
					Some((id, _)) => Ok(treeldr_build::layout::Description::Alias(id)),
					None => variant
						.layout
						.into_standard_description(source, target, vocabulary),
				}
			}
			_ => {
				let variants_id = target.try_create_list_with::<Error<F>, _, _>(
					vocabulary,
					self.variants,
					|variant, target, vocabulary| {
						let (variant, causes) = variant.into_parts();
						Ok(Caused::new(
							variant
								.into_variant(source, target, vocabulary, &causes)?
								.into_term(),
							causes.preferred().cloned(),
						))
					},
				)?;

				Ok(treeldr_build::layout::Description::Enum(variants_id))
			}
		}
	}
}

impl<F> PartialEq for IntersectedEnum<F> {
	fn eq(&self, other: &Self) -> bool {
		self.variants.len() == other.variants.len()
			&& self
				.variants
				.iter()
				.zip(&other.variants)
				.all(|(a, b)| a.inner() == b.inner())
	}
}

#[derive(Clone)]
pub struct IntersectedVariant<F> {
	name: MaybeSet<Name, F>,
	layout: IntersectedLayout<F>,
}

impl<F: Clone + Ord> IntersectedVariant<F> {
	pub fn into_variant(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
		causes: &Causes<F>,
	) -> Result<Id, Error<F>> {
		let layout = self.layout.into_layout(source, target, vocabulary)?;

		let id = Id::Blank(vocabulary.new_blank_label());
		target.declare_layout_variant(id, causes.preferred().cloned());

		let def = target.get_mut(id).unwrap().as_layout_variant_mut().unwrap();

		def.replace_name(self.name);
		def.replace_layout(layout.into());

		Ok(id)
	}

	pub fn matches(&self, other: &Self) -> bool {
		match (self.name.value(), other.name.value()) {
			(Some(a), Some(b)) => a == b,
			(None, None) => self.layout == other.layout,
			_ => false,
		}
	}

	pub fn intersected_with(
		self,
		other: Self,
		context: &Context<F, Descriptions>,
	) -> Result<Option<Self>, Error<F>> {
		let name = match (self.name.unwrap(), other.name.unwrap()) {
			(Some(a), Some(b)) => {
				let (a, causes) = a.into_parts();
				MaybeSet::new(a, causes.with(b.into_causes()))
			}
			(Some(a), _) => a.into(),
			(_, Some(b)) => b.into(),
			(None, None) => MaybeSet::default(),
		};

		let layout = self.layout.intersected_with(other.layout, context)?;

		if layout.is_never() {
			Ok(None)
		} else {
			Ok(Some(Self { name, layout }))
		}
	}
}

impl<F> PartialEq for IntersectedVariant<F> {
	fn eq(&self, other: &Self) -> bool {
		self.name.value() == other.name.value() && self.layout == other.layout
	}
}
