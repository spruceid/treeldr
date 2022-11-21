use std::collections::BTreeMap;

use locspan::{Meta, StrippedPartialEq, StrippedPartialOrd, StrippedOrd};
use locspan_derive::{StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd};
use rdf_types::{Generator, VocabularyMut};
use treeldr::{Id, metadata::Merge, Name, IriIndex, BlankIdIndex};

use crate::{Context, Error, ObjectAsRequiredId, utils::TryCollect, Single};

use super::Primitive;

#[derive(Debug, Clone)]
pub struct IdIntersection<M>(BTreeMap<Id, M>);

impl<M> IdIntersection<M> {
	pub fn new(Meta(id, m): Meta<Id, M>) -> Self {
		let mut map = BTreeMap::new();
		map.insert(id, m);

		Self(map)
	}

	pub fn iter(&self) -> impl '_ + Iterator<Item = Meta<Id, &M>> {
		self.0.iter().map(|(id, m)| Meta(*id, m))
	}
}

impl<M: Merge> IdIntersection<M> {
	pub fn insert(&mut self, Meta(value, meta): Meta<Id, M>) {
		use std::collections::btree_map::Entry;
		match self.0.entry(value) {
			Entry::Occupied(mut entry) => entry.get_mut().merge_with(meta),
			Entry::Vacant(entry) => {
				entry.insert(meta);
			}
		}
	}

	pub fn intersection(&self, other: &Self) -> Self where M: Clone + Merge {
		let mut result = self.clone();

		for v in other.iter() {
			result.insert(v.cloned_metadata())
		}

		result
	}

	pub fn intersect_with(&mut self, other: Self) where M: Clone + Merge {
		for (v, m) in other.0 {
			self.insert(Meta(v, m))
		}
	}

	pub fn prepare_layout<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut Vec<Id>,
		meta: M
	) -> Result<Id, Error<M>> where M: Clone {
		if self.0.len() == 1 {
			Ok(self.0.into_iter().next().unwrap().0)
		} else {
			let id = generator.next(vocabulary);

			let list_id = context.create_list_with(vocabulary, generator, self.0, |(layout_id, layout_meta), _, _| {
				Meta(layout_id.into_term(), layout_meta)
			})?;

			let node = context.declare_layout(id, meta.clone());
			node.as_layout_mut().intersection_of_mut().insert(Meta(list_id, meta));

			stack.push(id);
			Ok(id)
		}
	}
}

impl<M, N> PartialEq<IdIntersection<N>> for IdIntersection<M> {
	fn eq(&self, other: &IdIntersection<N>) -> bool {
		self.0.keys().eq(other.0.keys())
	}
}

impl<M> Eq for IdIntersection<M> {}

impl<M, N> PartialOrd<IdIntersection<N>> for IdIntersection<M> {
	fn partial_cmp(&self, other: &IdIntersection<N>) -> Option<std::cmp::Ordering> {
		self.0.keys().partial_cmp(other.0.keys())
	}
}

impl<M> Ord for IdIntersection<M> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.0.keys().cmp(other.0.keys())
	}
}

#[derive(Debug, Clone)]
pub struct BuiltDefinition<M> {
	pub desc: Single<super::Description, M>
}

#[derive(Debug, Clone)]
pub struct Definition<M> {
	/// Layout description.
	desc: Single<Description<M>, M>
}

impl<M> Definition<M> {
	pub fn from_id(
		context: &Context<M>,
		id: Id,
		meta: &M
	) -> Result<Option<Self>, Error<M>> where M: Clone + Merge {
		let node = context.require(id).map_err(|e| e.at(meta.clone()))?;
		let layout = node.require_layout(context).map_err(|e| e.at(meta.clone()))?;

		if layout.description().is_empty() {
			Ok(None)
		} else {
			let desc = layout.description().iter().map(Description::new).collect();

			Ok(Some(Self {
				desc
			}))
		}
	}

	pub fn add(&mut self, other: Self) where M: Merge {
		for d in other.desc {
			self.desc.insert(d)
		}
	}

	pub fn add_never(&mut self, meta: M) where M: Merge {
		self.desc.insert(Meta(Description::Never, meta))
	}

	pub fn intersect_with(&mut self, other: Self) where M: Clone + Merge {
		let desc = std::mem::take(&mut self.desc);
		for Meta(a, a_meta) in desc {
			for Meta(b, b_meta) in &other.desc {
				let mut c = a.clone();
				c.intersect_with(Meta(b.clone(), b_meta.clone()));
				self.desc.insert(Meta(c, a_meta.clone().merged_with(b_meta.clone())));
			}
		}
	}

	/// Compute the actual layout definition from the intersection definition.
	/// 
	/// Newly created intersection sub-layouts are added to the `stack`.
	pub fn build<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut Vec<Id>
	) -> Result<BuiltDefinition<M>, Error<M>> where M: Clone + Merge {
		let mut desc = Single::default();

		for Meta(d, meta) in self.desc {
			for built_d in d.build(vocabulary, generator, context, stack, meta)? {
				desc.insert(built_d)
			}
		}

		Ok(BuiltDefinition { desc })
	}
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			desc: Single::default()
		}
	}
}

/// Layout intersection description.
#[derive(Debug, Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd)]
#[locspan(ignore(M))]
pub enum Description<M> {
	Never,
	Primitive(#[locspan(stripped)] Primitive),
	Struct(#[locspan(stripped)] IdIntersection<M>),
	Reference(#[locspan(stripped)] IdIntersection<M>),
	Enum(#[locspan(stripped)] IdIntersection<M>),
	Required(#[locspan(stripped)] IdIntersection<M>),
	Option(#[locspan(stripped)] IdIntersection<M>),
	Set(#[locspan(stripped)] IdIntersection<M>),
	OneOrMany(#[locspan(stripped)] IdIntersection<M>),
	Array(#[locspan(stripped)] IdIntersection<M>),
	Alias(#[locspan(stripped)] IdIntersection<M>),
}

impl<M, N> PartialEq<Description<N>> for Description<M> {
	fn eq(&self, other: &Description<N>) -> bool {
		self.stripped_eq(other)
	}
}

impl<M> Eq for Description<M> {}

impl<M, N> PartialOrd<Description<N>> for Description<M> {
	fn partial_cmp(&self, other: &Description<N>) -> Option<std::cmp::Ordering> {
		self.stripped_partial_cmp(other)
	}
}

impl<M> Ord for Description<M> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.stripped_cmp(other)
	}
}

impl<M> Description<M> {
	pub fn new(Meta(desc, meta): Meta<&super::Description, &M>) -> Meta<Self, M> where M: Clone {
		let desc = match desc {
			super::Description::Alias(id) => Self::Alias(IdIntersection::new(Meta(*id, meta.clone()))),
			super::Description::Array(id) => Self::Array(IdIntersection::new(Meta(*id, meta.clone()))),
			super::Description::Enum(id) => Self::Enum(IdIntersection::new(Meta(*id, meta.clone()))),
			super::Description::Never => Self::Never,
			super::Description::OneOrMany(id) => Self::OneOrMany(IdIntersection::new(Meta(*id, meta.clone()))),
			super::Description::Option(id) => Self::Option(IdIntersection::new(Meta(*id, meta.clone()))),
			super::Description::Primitive(p) => Self::Primitive(*p),
			super::Description::Reference(id) => Self::Reference(IdIntersection::new(Meta(*id, meta.clone()))),
			super::Description::Required(id) => Self::Required(IdIntersection::new(Meta(*id, meta.clone()))),
			super::Description::Set(id) => Self::Set(IdIntersection::new(Meta(*id, meta.clone()))),
			super::Description::Struct(id) => Self::Struct(IdIntersection::new(Meta(*id, meta.clone()))),
		};

		Meta(desc, meta.clone())
	}

	pub fn intersect_with(&mut self, Meta(other, _): Meta<Description<M>, M>) where M: Clone + Merge {
		match (self, other) {
			(Self::Never, Self::Never) => (),
			(Self::Primitive(a), Self::Primitive(b)) if *a == b => (),
			(Self::Struct(a), Self::Struct(b)) => {
				a.intersect_with(b)
			},
			(Self::Reference(a), Self::Reference(b)) => {
				a.intersect_with(b)
			},
			(Self::Enum(a), Self::Enum(b)) => {
				a.intersect_with(b)
			},
			(Self::Required(a), Self::Required(b)) => {
				a.intersect_with(b)
			},
			(Self::Option(a), Self::Option(b)) => {
				a.intersect_with(b)
			},
			(Self::Set(a), Self::Set(b)) => {
				a.intersect_with(b)
			},
			(Self::OneOrMany(a), Self::OneOrMany(b)) => {
				a.intersect_with(b)
			},
			(Self::Array(a), Self::Alias(b)) => {
				a.intersect_with(b)
			},
			(this, _) => {
				*this = Self::Never
			}
		}
	}

	pub fn build<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut Vec<Id>,
		meta: M
	) -> Result<Single<super::Description, M>, Error<M>> where M: Clone + Merge {
		let mut desc = Single::default();
		
		match self {
			Self::Never => desc.insert(Meta(super::Description::Never, meta)),
			Self::Primitive(p) => desc.insert(Meta(super::Description::Primitive(p), meta)),
			Self::Struct(s) => {
				for s in struct_intersection(vocabulary, generator, context, stack, &s)? {
					match s {
						Some(id) => desc.insert(Meta(super::Description::Struct(id), meta.clone())),
						None => desc.insert(Meta(super::Description::Never, meta.clone()))
					}
				}
			}
			Self::Enum(e) => {
				for s in enum_intersection(vocabulary, generator, context, stack, &e)? {
					match s {
						Some(id) => desc.insert(Meta(super::Description::Enum(id), meta.clone())),
						None => desc.insert(Meta(super::Description::Never, meta.clone()))
					}
				}
			},
			Self::Reference(l) => desc.insert(Meta(super::Description::Reference(l.prepare_layout(vocabulary, generator, context, stack, meta.clone())?), meta)),
			Self::Required(l) => desc.insert(Meta(super::Description::Required(l.prepare_layout(vocabulary, generator, context, stack, meta.clone())?), meta)),
			Self::Option(l) => desc.insert(Meta(super::Description::Option(l.prepare_layout(vocabulary, generator, context, stack, meta.clone())?), meta)),
			Self::Set(l) => desc.insert(Meta(super::Description::Set(l.prepare_layout(vocabulary, generator, context, stack, meta.clone())?), meta)),
			Self::OneOrMany(l) => desc.insert(Meta(super::Description::OneOrMany(l.prepare_layout(vocabulary, generator, context, stack, meta.clone())?), meta)),
			Self::Array(l) => desc.insert(Meta(super::Description::Array(l.prepare_layout(vocabulary, generator, context, stack, meta.clone())?), meta)),
			Self::Alias(l) => desc.insert(Meta(super::Description::Alias(l.prepare_layout(vocabulary, generator, context, stack, meta.clone())?), meta)),
		};

		Ok(desc)
	}
}

fn fields_intersection<M: Clone + Merge>(
	fields: Option<&[Meta<Field<M>, M>]>,
	other_fields: &[Meta<Field<M>, M>]
) -> Result<Option<Vec<Meta<Field<M>, M>>>, Error<M>> {
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

pub fn struct_intersection<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>, M: Clone + Merge>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	context: &mut Context<M>,
	stack: &mut Vec<Id>,
	lists: &IdIntersection<M>
) -> Result<Vec<Option<Id>>, Error<M>> {
	let mut result: Vec<Option<Vec<Meta<Field<M>, M>>>> = Vec::new();

	for (i, Meta(list_id, meta)) in lists.iter().enumerate() {
		let list = context.require_list(list_id).map_err(|e| e.at(meta.clone()))?;

		let structs = list.try_fold(context, Vec::new(), |struct_, item| {
			let mut structs = Vec::new();

			for Meta(object, field_meta) in item {
				let mut struct_ = struct_.clone();
				let field_id = object.as_required_id(field_meta)?;
				struct_.push(Meta(Field::from_id(context, field_id, field_meta)?, field_meta.clone()));
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
					result.push(fields_intersection(a.as_deref(), b)?)
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

pub fn enum_intersection<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>, M: Clone + Merge>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	context: &mut Context<M>,
	stack: &mut Vec<Id>,
	lists: &IdIntersection<M>
) -> Result<Vec<Option<Id>>, Error<M>> {
	todo!()
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

	pub fn build<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut Vec<Id>,
		meta: M
	) -> Result<Id, Error<M>> where M: Clone + Merge {
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