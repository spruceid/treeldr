use super::{IntersectedLayout, IntersectedLayoutDescription};
use crate::build::{Descriptions, Error};
use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, Id, IriIndex, MetaOption, Name};
use treeldr_build::{Context, ObjectToId};

#[derive(Clone)]
pub struct IntersectedEnum<M> {
	variants: Vec<Meta<IntersectedVariant<M>, M>>,
}

impl<M: Clone> IntersectedEnum<M> {
	pub fn new(
		variants_id: Id,
		context: &Context<M, Descriptions>,
		causes: &M,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
		let mut variants = Vec::new();

		for variant_obj in context.require_list(variants_id, causes)?.iter(context) {
			let variant_obj = variant_obj?;
			let variant_id = variant_obj.as_required_id(variant_obj.metadata())?;
			let variant = context.require_layout_variant(variant_id, variant_obj.metadata())?;
			variants.push(Meta::new(
				IntersectedVariant {
					name: variant.name().cloned().into(),
					layout: IntersectedLayout::try_from_iter(
						variant.layout().as_ref().cloned(),
						context,
						variant_obj.metadata().clone(),
					)?,
				},
				variant.metadata().clone(),
			))
		}

		Ok(Self { variants })
	}

	pub fn is_included_in(&self, other: &Self) -> bool {
		let mut i = 0;
		'next_variant: for variant in &self.variants {
			while i < other.variants.len() {
				let other_variant = &other.variants[i];
				if variant.name.value() == other_variant.name.value()
					&& variant
						.layout
						.desc
						.is_included_in(&other_variant.layout.desc)
				{
					continue 'next_variant;
				}

				i += 1;
			}

			return false;
		}

		true
	}

	pub fn intersected_with(
		mut self,
		mut other: IntersectedEnum<M>,
		context: &Context<M, Descriptions>,
	) -> Result<IntersectedLayoutDescription<M>, Error<M>>
	where
		M: Merge,
	{
		let mut variants = std::mem::take(&mut self.variants);
		variants.reverse();
		other.variants.reverse();

		'next_variant: while !variants.is_empty() && !other.variants.is_empty() {
			if let Some(Meta(variant, causes)) = variants.pop() {
				while let Some(other_variant) = other.variants.pop() {
					if variant.matches(&other_variant) {
						let Meta(other_variant, other_causes) = other_variant;
						if let Some(intersected_variant) =
							variant.intersected_with(other_variant, context)?
						{
							self.variants.push(Meta::new(
								intersected_variant,
								causes.merged_with(other_causes),
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
				let variant = self.variants.into_iter().next().unwrap().into_value();
				match variant.layout.id() {
					Some((id, _)) => Ok(IntersectedLayoutDescription::Alias(id)),
					None => Ok(variant.layout.into_description().into_value()),
				}
			}
			_ => Ok(IntersectedLayoutDescription::Enum(self)),
		}
	}

	pub fn intersected_with_non_enum(
		mut self,
		other: IntersectedLayout<M>,
		context: &Context<M, Descriptions>,
	) -> Result<IntersectedLayoutDescription<M>, Error<M>>
	where
		M: Merge,
	{
		let variants = std::mem::take(&mut self.variants);

		for Meta(mut variant, causes) in variants {
			variant.layout = variant.layout.intersected_with(other.clone(), context)?;

			if variant.layout.has_id() {
				self.variants.push(Meta::new(variant, causes))
			}
		}

		match self.variants.len() {
			0 => Ok(IntersectedLayoutDescription::Never),
			1 => {
				let variant = self.variants.into_iter().next().unwrap().into_value();
				match variant.layout.id() {
					Some((id, _)) => Ok(IntersectedLayoutDescription::Alias(id)),
					None => Ok(variant.layout.into_description().into_value()),
				}
			}
			_ => Ok(IntersectedLayoutDescription::Enum(self)),
		}
	}

	pub fn into_standard_description<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<treeldr_build::layout::Description<M>, Error<M>>
	where
		M: Merge,
	{
		match self.variants.len() {
			0 => Ok(treeldr_build::layout::Description::Never),
			1 => {
				let variant = self.variants.into_iter().next().unwrap().into_value();
				match variant.layout.id() {
					Some((id, _)) => Ok(treeldr_build::layout::Description::Alias(id)),
					None => variant
						.layout
						.into_standard_description(source, target, vocabulary, generator),
				}
			}
			_ => {
				let variants_id = target.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					self.variants,
					|Meta(variant, meta), target, vocabulary, generator| {
						Ok(Meta(
							variant
								.into_variant(source, target, vocabulary, generator, &meta)?
								.into_term(),
							meta,
						))
					},
				)?;

				Ok(treeldr_build::layout::Description::Enum(variants_id))
			}
		}
	}
}

impl<M> PartialEq for IntersectedEnum<M> {
	fn eq(&self, other: &Self) -> bool {
		self.variants.len() == other.variants.len()
			&& self
				.variants
				.iter()
				.zip(&other.variants)
				.all(|(a, b)| a.value() == b.value())
	}
}

#[derive(Clone)]
pub struct IntersectedVariant<M> {
	name: MetaOption<Name, M>,
	layout: IntersectedLayout<M>,
}

impl<M: Clone> IntersectedVariant<M> {
	pub fn into_variant<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		causes: &M,
	) -> Result<Id, Error<M>>
	where
		M: Merge,
	{
		let layout = self
			.layout
			.into_layout(source, target, vocabulary, generator)?;

		let id = generator.next(vocabulary);
		target.declare_layout_variant(id, causes.clone());

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
		context: &Context<M, Descriptions>,
	) -> Result<Option<Self>, Error<M>>
	where
		M: Merge,
	{
		let name = match (self.name.unwrap(), other.name.unwrap()) {
			(Some(Meta(a, causes)), Some(b)) => {
				MetaOption::new(a, causes.merged_with(b.into_metadata()))
			}
			(Some(a), _) => a.into(),
			(_, Some(b)) => b.into(),
			(None, None) => MetaOption::default(),
		};

		let layout = self.layout.intersected_with(other.layout, context)?;

		if layout.is_never() {
			Ok(None)
		} else {
			Ok(Some(Self { name, layout }))
		}
	}
}

impl<M> PartialEq for IntersectedVariant<M> {
	fn eq(&self, other: &Self) -> bool {
		self.name.value() == other.name.value() && self.layout == other.layout
	}
}
