use std::collections::VecDeque;

use locspan::{Meta, StrippedEq, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};
use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, Id, IriIndex, Name};

use super::{build_lists, list::IntersectionListItem, list_intersection, IdIntersection};
use crate::{Context, Error, Single};

pub fn struct_intersection<
	V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>,
	M: Clone + Merge,
>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	context: &mut Context<M>,
	stack: &mut VecDeque<Id>,
	lists: &IdIntersection<M>,
) -> Result<Vec<Option<Id>>, Error<M>> {
	let list = list_intersection::<Field<M>, _>(context, lists)?;
	Ok(build_lists(vocabulary, generator, context, stack, list))
}

/// Field intersection.
#[derive(Clone)]
pub struct Field<M> {
	id: Option<Id>,
	name: Single<Name, M>,
	layout: Single<IdIntersection<M>, M>,
	prop: Single<Id, M>,
}

impl<M> StrippedPartialEq for Field<M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.name.stripped_eq(&other.name)
			&& self.prop.stripped_eq(&other.prop)
			&& self.layout.stripped_eq(&other.layout)
			&& self.id == other.id
	}
}

impl<M> StrippedEq for Field<M> {}

impl<M> StrippedPartialOrd for Field<M> {
	fn stripped_partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.stripped_cmp(other))
	}
}

impl<M> StrippedOrd for Field<M> {
	fn stripped_cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.name
			.stripped_cmp(&other.name)
			.then_with(|| self.prop.stripped_cmp(&other.prop))
			.then_with(|| self.layout.stripped_cmp(&other.layout))
			.then_with(|| self.id.cmp(&other.id))
	}
}

impl<M> Field<M> {
	pub fn from_id(context: &Context<M>, id: Id, meta: &M) -> Result<Self, Error<M>>
	where
		M: Clone + Merge,
	{
		let node = context.require(id).map_err(|e| e.at(meta.clone()))?;
		let field = node
			.require_layout_field(context)
			.map_err(|e| e.at(meta.clone()))?;

		Ok(Self {
			id: Some(id),
			name: node.as_component().name().clone(),
			layout: node
				.as_formatted()
				.format()
				.iter()
				.map(|id| {
					let meta = id.into_metadata().clone();
					Meta(IdIntersection::new(id.cloned()), meta)
				})
				.collect(),
			prop: field.property().clone(),
		})
	}
}

impl<M> Field<M> {
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

		let common_property = self
			.prop
			.iter()
			.any(|Meta(a, _)| other.prop.iter().any(|Meta(b, _)| a == b));
		let no_property = self.prop.is_empty() && other.prop.is_empty();

		if common_name || common_property || (no_name && no_property) {
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
				prop: self
					.prop
					.iter()
					.chain(&other.prop)
					.map(|p| p.cloned())
					.collect(),
			})
		} else {
			None
		}
	}
}

impl<M: Clone + Merge> IntersectionListItem<M> for Field<M> {
	fn get(context: &Context<M>, Meta(id, meta): Meta<Id, M>) -> Result<Meta<Self, M>, Error<M>> {
		Ok(Meta(Field::from_id(context, id, &meta)?, meta))
	}

	fn list_intersection(
		fields: Option<&[Meta<Self, M>]>,
		other_fields: &[Meta<Self, M>],
	) -> Result<Option<Vec<Meta<Self, M>>>, Error<M>> {
		match fields {
			Some(fields) => {
				let mut other_fields: Vec<_> = other_fields.iter().map(Some).collect();
				let mut result =
					Vec::with_capacity(std::cmp::max(fields.len(), other_fields.len()));

				'next_field: for field in fields {
					for other_field_opt in &mut other_fields {
						if let Some(other_field) = other_field_opt {
							if let Some(intersected_field) = field.intersected_with(other_field) {
								result.push(Meta(
									intersected_field,
									field
										.metadata()
										.clone()
										.merged_with(other_field.metadata().clone()),
								));
								*other_field_opt = None;
								continue 'next_field;
							}
						}
					}

					result.push(field.clone());
				}

				for other_field in other_fields.into_iter().flatten() {
					result.push(other_field.clone());
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
				*node.as_component_mut().name_mut() = self.name;
				*node.as_formatted_mut().format_mut() = layout;
				*node.as_layout_field_mut().property_mut() = self.prop;

				id
			}
		}
	}
}
