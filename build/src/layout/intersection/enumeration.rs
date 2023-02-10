use std::collections::VecDeque;

use locspan::{Meta, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};
use locspan_derive::{StrippedEq, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};
use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, Id, IriIndex, Name};

use super::{build_lists, list::IntersectionListItem, list_intersection, IdIntersection};
use crate::{Context, Error, Single};

#[derive(Debug, Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd)]
#[locspan(ignore(M))]
pub struct EnumIntersection<M> {
	#[locspan(stripped)]
	enums: IdIntersection<M>,

	#[locspan(stripped)]
	non_enums: IdIntersection<M>,
}

impl<M> EnumIntersection<M> {
	pub fn intersect_with(&mut self, other: Self)
	where
		M: Clone + Merge,
	{
		self.enums.intersect_with(other.enums)
	}

	pub fn intersect_with_non_enum(&mut self, other: Meta<Id, M>)
	where
		M: Clone + Merge,
	{
		self.non_enums.insert(other)
	}
}

impl<M> EnumIntersection<M> {
	pub fn new(id: Meta<Id, M>) -> Self {
		Self {
			enums: IdIntersection::new(id),
			non_enums: IdIntersection::empty(),
		}
	}
}

impl<M, N> PartialEq<EnumIntersection<N>> for EnumIntersection<M> {
	fn eq(&self, other: &EnumIntersection<N>) -> bool {
		self.stripped_eq(other)
	}
}

impl<M> Eq for EnumIntersection<M> {}

impl<M, N> PartialOrd<EnumIntersection<N>> for EnumIntersection<M> {
	fn partial_cmp(&self, other: &EnumIntersection<N>) -> Option<std::cmp::Ordering> {
		self.stripped_partial_cmp(other)
	}
}

impl<M> Ord for EnumIntersection<M> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.stripped_cmp(other)
	}
}

pub fn enum_intersection<
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	M: Clone + Merge,
>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	context: &mut Context<M>,
	stack: &mut VecDeque<Id>,
	inter: &EnumIntersection<M>,
) -> Result<Vec<Option<Id>>, Error<M>> {
	let mut lists = list_intersection::<Variant<M>, _>(context, &inter.enums)?;

	for list in &mut lists {
		non_enum_intersection(list.as_mut(), &inter.non_enums)
	}

	Ok(build_lists(vocabulary, generator, context, stack, lists))
}

/// Variant intersection.
#[derive(Clone)]
pub struct Variant<M> {
	id: Option<Id>,
	name: Single<Name, M>,
	layout: Single<IdIntersection<M>, M>,
}

impl<M> StrippedPartialEq for Variant<M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.name.stripped_eq(&other.name)
			&& self.layout.stripped_eq(&other.layout)
			&& self.id == other.id
	}
}

impl<M> locspan::StrippedEq for Variant<M> {}

impl<M> StrippedPartialOrd for Variant<M> {
	fn stripped_partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.stripped_cmp(other))
	}
}

impl<M> StrippedOrd for Variant<M> {
	fn stripped_cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.name
			.stripped_cmp(&other.name)
			.then_with(|| self.layout.stripped_cmp(&other.layout))
			.then_with(|| self.id.cmp(&other.id))
	}
}

impl<M> Variant<M> {
	pub fn from_id(context: &Context<M>, id: Id, meta: &M) -> Result<Self, Error<M>>
	where
		M: Clone + Merge,
	{
		let node = context.require(id).map_err(|e| e.at(meta.clone()))?;
		node.require_layout_variant(context)
			.map_err(|e| e.at(meta.clone()))?;

		Ok(Self {
			id: Some(id),
			name: node.as_component().name().clone_into_single(),
			layout: node
				.as_formatted()
				.format()
				.iter()
				.map(|id| {
					let meta = id.value.into_metadata().clone();
					Meta(IdIntersection::new(id.value.cloned()), meta)
				})
				.collect(),
		})
	}
}

impl<M> Variant<M> {
	pub fn intersected_with(&self, other: &Self) -> Option<Self>
	where
		M: Clone + Merge,
	{
		if let (Some(a), Some(b)) = (self.id, other.id) {
			if a == b {
				return Some(self.clone());
			}
		}

		let common_name = self
			.name
			.iter()
			.any(|Meta(a, _)| other.name.iter().any(|Meta(b, _)| a == b));
		let no_name = self.name.is_empty() && other.name.is_empty();

		if common_name || no_name {
			let mut layout = Single::default();
			for Meta(a, a_meta) in &self.layout {
				for Meta(b, b_meta) in &other.layout {
					layout.insert(Meta(
						a.intersection(b),
						a_meta.clone().merged_with(b_meta.clone()),
					))
				}
			}

			Some(Self {
				id: None,
				name: self
					.name
					.iter()
					.chain(&other.name)
					.map(|n| n.cloned())
					.collect(),
				layout,
			})
		} else {
			None
		}
	}
}

impl<M: Clone + Merge> IntersectionListItem<M> for Variant<M> {
	fn get(context: &Context<M>, Meta(id, meta): Meta<Id, M>) -> Result<Meta<Self, M>, Error<M>> {
		Ok(Meta(Variant::from_id(context, id, &meta)?, meta))
	}

	fn list_intersection(
		variants: Option<&[Meta<Self, M>]>,
		other_variants: &[Meta<Self, M>],
	) -> Result<Option<Vec<Meta<Self, M>>>, Error<M>> {
		match variants {
			Some(variants) => {
				let mut other_variants: Vec<_> = other_variants.iter().map(Some).collect();
				let mut result =
					Vec::with_capacity(std::cmp::max(variants.len(), other_variants.len()));

				'next_variant: for variant in variants {
					for other_variant_opt in &mut other_variants {
						if let Some(other_variant) = other_variant_opt {
							if let Some(intersected_variant) =
								variant.intersected_with(other_variant)
							{
								result.push(Meta(
									intersected_variant,
									variant
										.metadata()
										.clone()
										.merged_with(other_variant.metadata().clone()),
								));
								*other_variant_opt = None;
								continue 'next_variant;
							}
						}
					}
				}

				result.sort_by(|a, b| a.value().stripped_cmp(b.value()));

				Ok(Some(result))
			}
			None => Ok(None),
		}
	}

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut VecDeque<Id>,
		meta: M,
	) -> Id {
		match self.id {
			Some(id) => id,
			None => {
				let mut layout = Single::default();
				for Meta(layout_id, layout_meta) in self.layout {
					layout.insert(Meta(
						layout_id.prepare_layout(
							vocabulary,
							generator,
							context,
							stack,
							layout_meta.clone(),
						),
						layout_meta,
					))
				}

				let id = generator.next(vocabulary);

				let node = context.declare_layout_field(id, meta);
				*node.as_component_mut().name_mut() = self.name.into_functional_property_value();
				*node.as_formatted_mut().format_mut() = layout.into_functional_property_value();

				id
			}
		}
	}
}

fn non_enum_intersection<M>(
	variants: Option<&mut Vec<Meta<Variant<M>, M>>>,
	other: &IdIntersection<M>,
) where
	M: Clone + Merge,
{
	if let Some(variants) = variants {
		for v in variants {
			let layout = std::mem::take(&mut v.layout);

			for Meta(mut ids, m) in layout {
				ids.intersect_with(other.clone());
				v.layout.insert(Meta(ids, m))
			}
		}
	}
}
