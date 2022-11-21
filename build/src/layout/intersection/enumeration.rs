use std::collections::VecDeque;

use locspan::{Meta, StrippedPartialEq, StrippedPartialOrd, StrippedOrd};
use locspan_derive::{StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd};
use rdf_types::{VocabularyMut, Generator};
use treeldr::{metadata::Merge, IriIndex, BlankIdIndex, Id, Name};

use crate::{Error, Context, Single};
use super::{IdIntersection, list::IntersectionListItem, list_intersection, build_lists};

#[derive(Debug, Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd)]
#[locspan(ignore(M))]
pub struct EnumIntersection<M> {
	#[locspan(stripped)]
	enums: IdIntersection<M>,

	#[locspan(stripped)]
	non_enums: IdIntersection<M>
}

impl<M> EnumIntersection<M> {
	pub fn intersect_with(&mut self, other: Self) where M: Clone + Merge {
		self.enums.intersect_with(other.enums)
	}
}

impl<M> EnumIntersection<M> {
	pub fn new(id: Meta<Id, M>) -> Self {
		Self { enums: IdIntersection::new(id), non_enums: IdIntersection::empty() }
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

pub fn enum_intersection<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>, M: Clone + Merge>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	context: &mut Context<M>,
	stack: &mut VecDeque<Id>,
	inter: &EnumIntersection<M>
) -> Result<Vec<Option<Id>>, Error<M>> {
	let mut lists = list_intersection::<Variant<M>, _>(context, &inter.enums)?;

	for list in &mut lists {
		non_enum_intersection(list.as_mut(), &inter.non_enums)
	}

	build_lists(vocabulary, generator, context, stack, lists)
}

/// Variant intersection.
#[derive(Clone)]
pub struct Variant<M> {
	id: Option<Id>,
	name: Single<Name, M>,
	layout: Single<IdIntersection<M>, M>
}

impl<M> Variant<M> {
	pub fn from_id(context: &Context<M>, id: Id, meta: &M) -> Result<Self, Error<M>> where M: Clone + Merge {
		let node = context.require(id).map_err(|e| e.at(meta.clone()))?;
		node.require_layout_variant(context).map_err(|e| e.at(meta.clone()))?;

		Ok(Self {
			id: Some(id),
			name: node.as_component().name().clone(),
			layout: node.as_formatted().format().iter().map(|id| {
				let meta = id.into_metadata().clone();
				Meta(IdIntersection::new(id.cloned()), meta)
			}).collect()
		})
	}
}

impl<M> Variant<M> {
	pub fn matches(&self, other: &Self) -> bool {
		let common_name = self.name.iter().any(|Meta(a, _)| other.name.iter().any(|Meta(b, _)| a == b));
		let no_name = self.name.is_empty() && other.name.is_empty();

		common_name || no_name
	}

	pub fn intersected_with(&self, other: &Self) -> Option<Self> where M: Clone + Merge {
		if let (Some(a), Some(b)) = (self.id, other.id) {
			if a == b {
				return Some(self.clone())
			}
		}

		let common_name = self.name.iter().any(|Meta(a, _)| other.name.iter().any(|Meta(b, _)| a == b));
		let no_name = self.name.is_empty() && other.name.is_empty();

		if common_name || no_name {
			let mut layout = Single::default();
			for Meta(a, a_meta) in &self.layout {
				for Meta(b, b_meta) in &other.layout {
					layout.insert(Meta(a.intersection(b), a_meta.clone().merged_with(b_meta.clone())))
				}
			}

			Some(Self {
				id: None,
				name: self.name.iter().chain(&other.name).map(|n| n.cloned()).collect(),
				layout
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

	fn list_intersection(variants: Option<&[Meta<Self, M>]>, other_variants: &[Meta<Self, M>]) -> Result<Option<Vec<Meta<Self, M>>>, Error<M>> {
		match variants {
			Some(variants) => {
				let mut result = Vec::new();
				let mut variants = variants.to_vec();
				let mut other_variants = other_variants.to_vec();
				variants.reverse();
				other_variants.reverse();
	
				'next_variant: while !variants.is_empty() && !other_variants.is_empty() {
					if let Some(Meta(variant, causes)) = variants.pop() {
						while let Some(other_variant) = other_variants.pop() {
							if variant.matches(&other_variant) {
								let Meta(other_variant, other_causes) = other_variant;
								if let Some(intersected_variant) =
									variant.intersected_with(&other_variant)
								{
									result.push(Meta::new(
										intersected_variant,
										causes.merged_with(other_causes),
									))
								}
		
								continue 'next_variant;
							} else {
								for after_variant in &variants {
									if after_variant.matches(&other_variant) {
										for j in 0..other_variants.len() {
											if variant.matches(&other_variants[j]) {
												panic!("unaligned layouts")
											}
										}
		
										other_variants.push(other_variant);
										continue 'next_variant;
									}
								}
							}
						}
					}
				}
		
				Ok(Some(result))
			}
			None => Ok(None)
		}
	}

	fn build<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut VecDeque<Id>,
		meta: M
	) -> Result<Id, Error<M>> {
		match self.id {
			Some(id) => Ok(id),
			None => {
				let mut layout = Single::default();
				for Meta(layout_id, layout_meta) in self.layout {
					layout.insert(Meta(layout_id.prepare_layout(vocabulary, generator, context, stack, layout_meta.clone())?, layout_meta))
				}

				let id = generator.next(vocabulary);

				let node = context.declare_layout_field(id, meta);
				*node.as_component_mut().name_mut() = self.name;
				*node.as_formatted_mut().format_mut() = layout;

				Ok(id)
			}
		}
	}
}

fn non_enum_intersection<M>(
	variants: Option<&mut Vec<Meta<Variant<M>, M>>>,
	other: &IdIntersection<M>
) where M: Clone + Merge {
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