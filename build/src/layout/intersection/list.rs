use std::collections::VecDeque;

use locspan::Meta;
use rdf_types::Generator;
use treeldr::{metadata::Merge, vocab::TldrVocabulary, Id, PropertyValueRef};

use crate::{utils::TryCollect, Context, Error, MetaValueExt};

use super::IdIntersection;

pub trait IntersectionListItem<M>: Clone {
	fn get(context: &Context<M>, id: Meta<Id, M>) -> Result<Meta<Self, M>, Error<M>>;

	fn list_intersection(
		a: Option<&[Meta<Self, M>]>,
		b: &[Meta<Self, M>],
	) -> Result<Option<Vec<Meta<Self, M>>>, Error<M>>;

	fn build(
		self,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
		context: &mut Context<M>,
		stack: &mut VecDeque<Id>,
		meta: M,
	) -> Id;
}

pub type IntersectedListItem<T, M> = Option<Vec<Meta<T, M>>>;

pub fn list_intersection<T: IntersectionListItem<M>, M: Clone + Merge>(
	context: &mut Context<M>,
	lists: &IdIntersection<M>,
) -> Result<Vec<IntersectedListItem<T, M>>, Error<M>> {
	let mut result: Vec<IntersectedListItem<T, M>> = Vec::new();

	for (i, Meta(list_id, meta)) in lists.iter().enumerate() {
		let list = context
			.require_list(list_id)
			.map_err(|e| e.at(meta.clone()))?;

		let structs = list.try_fold(context, Vec::new(), |struct_, item| {
			let mut structs = Vec::new();

			for PropertyValueRef { value, .. } in item {
				let mut struct_ = struct_.clone();
				let Meta(field_id, field_meta) = value.cloned().into_expected_id()?;
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

	Ok(result)
}

pub fn build_lists<T: IntersectionListItem<M>, M: Clone + Merge>(
	vocabulary: &mut TldrVocabulary,
	generator: &mut impl Generator<TldrVocabulary>,
	context: &mut Context<M>,
	stack: &mut VecDeque<Id>,
	result: Vec<Option<Vec<Meta<T, M>>>>,
) -> Vec<Option<Id>> {
	result
		.into_iter()
		.map(|fields| {
			fields.map(|fields| {
				context.create_list_with(
					vocabulary,
					generator,
					fields,
					|Meta(f, m), context, vocabulary, generator| {
						Meta(
							f.build(vocabulary, generator, context, stack, m.clone())
								.into_term(),
							m,
						)
					},
				)
			})
		})
		.collect()
}
