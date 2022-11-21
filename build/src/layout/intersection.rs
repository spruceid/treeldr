use std::collections::VecDeque;

use locspan::{Meta, StrippedPartialEq, StrippedPartialOrd, StrippedOrd};
use locspan_derive::{StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd};
use rdf_types::{Generator, VocabularyMut};
use treeldr::{Id, metadata::Merge, IriIndex, BlankIdIndex};

use crate::{Context, Error, Single};

mod id;
mod list;
mod structure;
mod enumeration;

pub use id::IdIntersection;
pub(crate) use list::{list_intersection, build_lists};
use structure::struct_intersection;
use enumeration::enum_intersection;

use self::enumeration::EnumIntersection;

use super::Primitive;

#[derive(Debug, Clone)]
pub struct Definition<M> {
	id: Meta<Id, M>,

	/// Layout description.
	desc: Single<Description<M>, M>
}

impl<M> Definition<M> {
	pub fn new(id: Meta<Id, M>) -> Self {
		Self { id, desc: Single::default() }
	}

	pub fn from_id(
		context: &Context<M>,
		id: Meta<Id, M>
	) -> Result<Option<Self>, Error<M>> where M: Clone + Merge {
		let node = context.require(*id).map_err(|e| e.at(id.metadata().clone()))?;
		let layout = node.require_layout(context).map_err(|e| e.at(id.metadata().clone()))?;

		if layout.description().is_empty() {
			Ok(None)
		} else {
			let desc = layout.description().iter().map(Description::new).collect();

			Ok(Some(Self {
				id,
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
				let c = if a.is_enum() && !b.is_enum() {
					let mut c = a.clone();
					c.intersect_enum_with_non_enum(other.id.clone());
					c
				} else if !a.is_enum() && b.is_enum() {
					let mut c = b.clone();
					c.intersect_enum_with_non_enum(self.id.clone());
					c
				} else {
					let mut c = a.clone();
					c.intersect_with(Meta(b.clone(), b_meta.clone()));
					c
				};

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
		stack: &mut VecDeque<Id>
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

#[derive(Debug, Clone)]
pub struct BuiltDefinition<M> {
	pub desc: Single<super::Description, M>
}

/// Layout intersection description.
#[derive(Debug, Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd)]
#[locspan(ignore(M))]
pub enum Description<M> {
	Never,
	Primitive(#[locspan(stripped)] Primitive),
	Struct(#[locspan(stripped)] IdIntersection<M>),
	Reference(#[locspan(stripped)] IdIntersection<M>),
	Enum(#[locspan(stripped)] EnumIntersection<M>),
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
			super::Description::Enum(id) => Self::Enum(EnumIntersection::new(Meta(*id, meta.clone()))),
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

	pub fn is_enum(&self) -> bool {
		matches!(self, Self::Enum(_))
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

	pub fn intersect_enum_with_non_enum(&mut self, id: Meta<Id, M>) where M: Clone + Merge {
		match self {
			Self::Enum(e) => e.intersect_with_non_enum(id),
			_ => panic!("not an enum")
		}
	}

	pub fn build<V: VocabularyMut<Iri=IriIndex, BlankId=BlankIdIndex>>(
		self,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		context: &mut Context<M>,
		stack: &mut VecDeque<Id>,
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