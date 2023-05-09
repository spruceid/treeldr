use std::collections::VecDeque;

use locspan::{Meta, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};
use locspan_derive::{StrippedEq, StrippedOrd, StrippedPartialEq, StrippedPartialOrd};
use rdf_types::{Generator, VocabularyMut};
use treeldr::{metadata::Merge, BlankIdIndex, Id, IriIndex};

use crate::{Context, Error, Single};

mod enumeration;
mod id;
mod list;
mod structure;

use enumeration::enum_intersection;
pub use id::IdIntersection;
pub(crate) use list::{build_lists, list_intersection};
use structure::struct_intersection;

use self::enumeration::EnumIntersection;

#[derive(Debug, Clone)]
pub struct Definition<M> {
	id: Meta<Id, M>,

	/// Layout description.
	desc: Single<Description<M>, M>,
}

impl<M> Definition<M> {
	pub fn new(id: Meta<Id, M>) -> Self {
		Self {
			id,
			desc: Single::default(),
		}
	}

	pub fn from_id(context: &Context<M>, id: Meta<Id, M>) -> Result<Option<Self>, Error<M>>
	where
		M: Clone + Merge,
	{
		let node = context
			.require(*id)
			.map_err(|e| e.at(id.metadata().clone()))?;
		let layout = node
			.require_layout(context)
			.map_err(|e| e.at(id.metadata().clone()))?;

		if layout.description().is_empty() {
			Ok(None)
		} else {
			let desc = layout.description().iter().map(Description::new).collect();

			Ok(Some(Self { id, desc }))
		}
	}

	pub fn add(&mut self, other: Self)
	where
		M: Merge,
	{
		for d in other.desc {
			self.desc.insert(d)
		}
	}

	pub fn add_never(&mut self, meta: M)
	where
		M: Merge,
	{
		self.desc.insert(Meta(Description::Never, meta))
	}

	pub fn intersect_with(&mut self, other: Self)
	where
		M: Clone + Merge,
	{
		let desc = std::mem::take(&mut self.desc);
		for Meta(a, a_meta) in desc {
			for Meta(b, b_meta) in &other.desc {
				let c = if a.is_enum() && !b.is_enum() {
					let mut c = a.clone();
					c.intersect_enum_with_non_enum(other.id.clone());
					c
				} else if !a.is_enum() && b.is_enum() {
					let mut c = b.clone();
					c.intersect_enum_with_non_enum(self.id.clone());
					c
				} else {
					a.clone().intersected_with(Meta(b.clone(), b_meta.clone()))
				};

				self.desc
					.insert(Meta(c, a_meta.clone().merged_with(b_meta.clone())));
			}
		}
	}

	/// Compute the actual layout definition from the intersection definition.
	///
	/// Newly created intersection sub-layouts are added to the `stack`.
	pub fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut VecDeque<Id>,
	) -> Result<BuiltDefinition<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let mut desc = Single::default();

		for Meta(d, meta) in self.desc {
			for built_d in d.build(vocabulary, generator, context, stack, meta)? {
				desc.insert(built_d)
			}
		}

		Ok(BuiltDefinition { desc })
	}
}

#[derive(Debug, Clone)]
pub struct BuiltDefinition<M> {
	pub desc: Single<super::BaseDescriptionBinding, M>,
}

/// Layout intersection description.
#[derive(Debug, Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd)]
#[locspan(ignore(M))]
pub enum Description<M> {
	Never,
	DerivedFrom(#[locspan(stripped)] IdIntersection<M>),
	Struct(#[locspan(stripped)] IdIntersection<M>),
	Reference(#[locspan(stripped)] IdIntersection<M>),
	Enum(#[locspan(stripped)] EnumIntersection<M>),
	Required(#[locspan(stripped)] IdIntersection<M>),
	Option(#[locspan(stripped)] IdIntersection<M>),
	Set(#[locspan(stripped)] IdIntersection<M>),
	Map(#[locspan(stripped)] IdIntersection<M>),
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
	pub fn new(Meta(desc, meta): Meta<super::DescriptionBinding, &M>) -> Meta<Self, M>
	where
		M: Clone,
	{
		let desc = match desc {
			super::DescriptionBinding::Alias(_, id) => {
				Self::Alias(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::Array(_, id) => {
				Self::Array(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::Enum(_, id) => {
				Self::Enum(EnumIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::OneOrMany(_, id) => {
				Self::OneOrMany(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::Option(_, id) => {
				Self::Option(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::DerivedFrom(_, id) => {
				Self::DerivedFrom(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::Reference(_, id) => {
				Self::Reference(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::Required(_, id) => {
				Self::Required(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::Set(_, id) => {
				Self::Set(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::Map(_, id) => {
				Self::Map(IdIntersection::new(Meta(id, meta.clone())))
			}
			super::DescriptionBinding::Struct(_, id) => {
				Self::Struct(IdIntersection::new(Meta(id, meta.clone())))
			}
		};

		Meta(desc, meta.clone())
	}

	pub fn is_enum(&self) -> bool {
		matches!(self, Self::Enum(_))
	}

	pub fn intersected_with(self, Meta(other, _): Meta<Description<M>, M>) -> Self
	where
		M: Clone + Merge,
	{
		match (self, other) {
			(Self::Never, Self::Never) => Self::Never,
			(Self::DerivedFrom(a), Self::DerivedFrom(b)) if a == b => Self::DerivedFrom(a),
			(Self::Alias(a), Self::Alias(b)) if a == b => Self::Alias(a),
			(Self::Struct(a), Self::Struct(b)) => Self::Struct(a.intersected_with(b)),
			(Self::Reference(a), Self::Reference(b)) => Self::Reference(a.intersected_with(b)),
			(Self::Enum(mut a), Self::Enum(b)) => {
				a.intersect_with(b);
				Self::Enum(a)
			}
			(Self::Required(a), Self::Required(b) | Self::Option(b) | Self::OneOrMany(b))
			| (Self::Option(b) | Self::OneOrMany(b), Self::Required(a)) => {
				Self::Required(a.intersected_with(b))
			}
			(Self::Option(a), Self::Option(b) | Self::OneOrMany(b))
			| (Self::OneOrMany(b), Self::Option(a)) => Self::Option(a.intersected_with(b)),
			(Self::Array(a), Self::Array(b) | Self::OneOrMany(b))
			| (Self::OneOrMany(a), Self::Array(b)) => Self::Array(a.intersected_with(b)),
			(Self::Set(a), Self::Set(b) | Self::OneOrMany(b))
			| (Self::OneOrMany(a), Self::Set(b)) => Self::Set(a.intersected_with(b)),
			(Self::OneOrMany(a), Self::OneOrMany(b)) => Self::OneOrMany(a.intersected_with(b)),
			_ => Self::Never,
		}
	}

	pub fn intersect_enum_with_non_enum(&mut self, id: Meta<Id, M>)
	where
		M: Clone + Merge,
	{
		match self {
			Self::Enum(e) => e.intersect_with_non_enum(id),
			_ => panic!("not an enum"),
		}
	}

	pub fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut VecDeque<Id>,
		meta: M,
	) -> Result<Single<super::BaseDescriptionBinding, M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let mut desc = Single::default();

		match self {
			Self::Never => (),
			Self::DerivedFrom(p) => desc.insert(Meta(
				super::BaseDescriptionBinding::DerivedFrom(p.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
			Self::Struct(s) => {
				for id in struct_intersection(vocabulary, generator, context, stack, &s)?
					.into_iter()
					.flatten()
				{
					desc.insert(Meta(
						super::BaseDescriptionBinding::Struct(id),
						meta.clone(),
					))
				}
			}
			Self::Enum(e) => {
				for id in enum_intersection(vocabulary, generator, context, stack, &e)?
					.into_iter()
					.flatten()
				{
					desc.insert(Meta(super::BaseDescriptionBinding::Enum(id), meta.clone()))
				}
			}
			Self::Reference(l) => desc.insert(Meta(
				super::BaseDescriptionBinding::Reference(l.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
			Self::Required(l) => desc.insert(Meta(
				super::BaseDescriptionBinding::Required(l.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
			Self::Option(l) => desc.insert(Meta(
				super::BaseDescriptionBinding::Option(l.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
			Self::Set(l) => desc.insert(Meta(
				super::BaseDescriptionBinding::Set(l.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
			Self::Map(l) => desc.insert(Meta(
				super::BaseDescriptionBinding::Map(l.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
			Self::OneOrMany(l) => desc.insert(Meta(
				super::BaseDescriptionBinding::OneOrMany(l.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
			Self::Array(l) => desc.insert(Meta(
				super::BaseDescriptionBinding::Array(l.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
			Self::Alias(l) => desc.insert(Meta(
				super::BaseDescriptionBinding::Alias(l.prepare_layout(
					vocabulary,
					generator,
					context,
					stack,
					meta.clone(),
				)),
				meta,
			)),
		};

		Ok(desc)
	}
}
