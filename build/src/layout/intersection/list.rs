use std::collections::VecDeque;

use locspan::Meta;
use rdf_types::{VocabularyMut, Generator};
use treeldr::{metadata::Merge, IriIndex, BlankIdIndex, Id};

use crate::{Context, Error, ObjectAsRequiredId, utils::TryCollect};

use super::IdIntersection;

pub trait IntersectionListItem<M>: Clone {
	fn get(context: &Context<M>, id: Meta<Id, M>) -> Result<Meta<Self, M>, Error<M>>;

	fn list_intersection(a: Option<&[Meta<Self, M>]>, b: &[Meta<Self, M>]) -> Result<Option<Vec<Meta<Self, M>>>, Error<M>>;

	fn build<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut VecDeque<Id>,
		meta: M
	) -> Result<Id, Error<M>>;
}

pub fn list_intersection<T: IntersectionListItem<M>, V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>, M: Clone + Merge>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	context: &mut Context<M>,
	stack: &mut VecDeque<Id>,
	lists: &IdIntersection<M>,
) -> Result<Vec<Option<Id>>, Error<M>> {
	let mut result: Vec<Option<Vec<Meta<T, M>>>> = Vec::new();

	for (i, Meta(list_id, meta)) in lists.iter().enumerate() {
		let list = context.require_list(list_id).map_err(|e| e.at(meta.clone()))?;

		let structs = list.try_fold(context, Vec::new(), |struct_, item| {
			let mut structs = Vec::new();

			for Meta(object, field_meta) in item {
				let mut struct_ = struct_.clone();
				let field_id = object.as_required_id(field_meta)?;
				struct_.push(T::get(context, Meta(field_id, field_meta.clone()))?);
				structs.push(struct_);
			}

			Ok(structs)
		});

		if i == 0 {
			for struct_ in structs {
				result.push(Some(struct_?))
			}
		} else {
			let a_structs = std::mem::take(&mut result);
			let b_structs: Vec<_> = structs.try_collect()?;

			for a in &a_structs {
				for b in &b_structs {
					result.push(T::list_intersection(a.as_deref(), b)?)
				}
			}
		}
	}

	result.into_iter().map(|fields| {
		fields.map(|fields| {
			let built_fields: Vec<_> = fields
				.into_iter()
				.map(|Meta(f, m)| Ok(Meta(f.build(vocabulary, generator, context, stack, m.clone())?.into_term(), m)))
				.try_collect()?;

			Ok(context.create_list(vocabulary, generator, built_fields))
		}).transpose()
	}).try_collect()
}