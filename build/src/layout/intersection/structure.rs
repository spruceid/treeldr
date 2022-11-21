use std::collections::VecDeque;

use locspan::Meta;
use rdf_types::{VocabularyMut, Generator};
use treeldr::{metadata::Merge, IriIndex, BlankIdIndex, Id, Name};

use crate::{Error, Context, Single};
use super::{IdIntersection, list::IntersectionListItem, list_intersection};

pub fn struct_intersection<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>, M: Clone + Merge>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	context: &mut Context<M>,
	stack: &mut VecDeque<Id>,
	lists: &IdIntersection<M>
) -> Result<Vec<Option<Id>>, Error<M>> {
	list_intersection::<Field<M>, _, _>(vocabulary, generator, context, stack, lists)
}

/// Field intersection.
#[derive(Clone)]
pub struct Field<M> {
	id: Option<Id>,
	name: Single<Name, M>,
	layout: Single<IdIntersection<M>, M>,
	prop: Single<Id, M>
}

impl<M> Field<M> {
	pub fn from_id(context: &Context<M>, id: Id, meta: &M) -> Result<Self, Error<M>> where M: Clone + Merge {
		let node = context.require(id).map_err(|e| e.at(meta.clone()))?;
		let field = node.require_layout_field(context).map_err(|e| e.at(meta.clone()))?;

		Ok(Self {
			id: Some(id),
			name: node.as_component().name().clone(),
			layout: node.as_formatted().format().iter().map(|id| {
				let meta = id.into_metadata().clone();
				Meta(IdIntersection::new(id.cloned()), meta)
			}).collect(),
			prop: field.property().clone()
		})
	}
}

impl<M> Field<M> {
	pub fn matches(&self, other: &Self) -> bool {
		let common_name = self.name.iter().any(|Meta(a, _)| other.name.iter().any(|Meta(b, _)| a == b));
		let no_name = self.name.is_empty() && other.name.is_empty();

		let common_property = self.prop.iter().any(|Meta(a, _)| other.prop.iter().any(|Meta(b, _)| a == b));
		let no_property = self.prop.is_empty() && other.prop.is_empty();

		common_name || common_property || (no_name && no_property)
	}

	pub fn intersected_with(&self, other: &Self) -> Option<Self> where M: Clone + Merge {
		if let (Some(a), Some(b)) = (self.id, other.id) {
			if a == b {
				return Some(self.clone())
			}
		}

		let common_name = self.name.iter().any(|Meta(a, _)| other.name.iter().any(|Meta(b, _)| a == b));
		let no_name = self.name.is_empty() && other.name.is_empty();

		let common_property = self.prop.iter().any(|Meta(a, _)| other.prop.iter().any(|Meta(b, _)| a == b));
		let no_property = self.prop.is_empty() && other.prop.is_empty();

		if (common_property || no_property) && (common_name || no_name) {
			let mut layout = Single::default();
			for Meta(a, a_meta) in &self.layout {
				for Meta(b, b_meta) in &other.layout {
					layout.insert(Meta(a.intersection(b), a_meta.clone().merged_with(b_meta.clone())))
				}
			}

			Some(Self {
				id: None,
				name: self.name.iter().chain(&other.name).map(|n| n.cloned()).collect(),
				layout,
				prop: self.prop.iter().chain(&other.prop).map(|p| p.cloned()).collect()
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

	fn list_intersection(fields: Option<&[Meta<Self, M>]>, other_fields: &[Meta<Self, M>]) -> Result<Option<Vec<Meta<Self, M>>>, Error<M>> {
		match fields {
			Some(fields) => {
				let mut result = Vec::new();
				let mut fields = fields.to_vec();
				let mut other_fields = other_fields.to_vec();
				fields.reverse();
				other_fields.reverse();
	
				'next_field: while !fields.is_empty() || !other_fields.is_empty() {
					match fields.pop() {
						Some(field) => {
							let Meta(field, causes) = field;
							while let Some(other_field) = other_fields.pop() {
								if field.matches(&other_field) {
									let Meta(other_field, other_causes) = other_field;
									match field.intersected_with(&other_field) {
										Some(intersected_field) => result.push(Meta(
											intersected_field,
											causes.merged_with(other_causes),
										)),
										None => return Ok(None),
									}
	
									continue 'next_field;
								} else {
									for after_field in &fields {
										if after_field.matches(&other_field) {
											for j in 0..other_fields.len() {
												if field.matches(&other_fields[j]) {
													panic!("unaligned layouts")
												}
											}
										}
									}
	
									result.push(other_field);
								}
							}
	
							result.push(Meta(field, causes));
						}
						None => {
							result.push(other_fields.pop().unwrap());
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
				*node.as_layout_field_mut().property_mut() = self.prop;

				Ok(id)
			}
		}
	}
}