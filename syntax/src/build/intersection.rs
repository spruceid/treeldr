use super::{Descriptions, Error, LayoutDescription, LayoutRestrictedField, LocalError};
use derivative::Derivative;
use locspan::Loc;
use std::collections::BTreeMap;
use treeldr::{
	layout::{literal::RegExp, Primitive},
	Causes, Id, MaybeSet, Vocabulary, WithCauses,
};
use treeldr_build::{layout::Array, Context, ObjectToId};

mod enumeration;
mod structure;

pub use enumeration::*;
pub use structure::*;

#[derive(Clone)]
pub struct IntersectedLayout<F> {
	ids: BTreeMap<Id, Causes<F>>,
	desc: WithCauses<IntersectedLayoutDescription<F>, F>,
}

impl<F> PartialEq for IntersectedLayout<F> {
	fn eq(&self, other: &Self) -> bool {
		self.ids.keys().any(|id| other.ids.contains_key(id))
			|| (self.ids.is_empty()
				&& other.ids.is_empty()
				&& self.desc.inner() == other.desc.inner())
	}
}

#[derive(Clone, Derivative)]
#[derivative(PartialEq(bound = ""))]
pub enum IntersectedLayoutDescription<F> {
	Never,
	Primitive(Primitive),
	Struct(IntersectedStruct<F>),
	Reference(Id),
	Literal(RegExp),
	Enum(IntersectedEnum<F>),
	Set(Id),
	Array(Array<F>),
	Alias(Id),
}

impl<F: Clone + Ord> IntersectedLayout<F> {
	pub fn try_from_iter<I: IntoIterator<Item = WithCauses<Id, F>>>(
		ids: I,
		context: &Context<F, Descriptions>,
		causes: Causes<F>,
	) -> Result<Self, Error<F>> {
		let mut ids = ids.into_iter();
		match ids.next() {
			Some(first_id) => {
				let mut result = IntersectedLayout::new(first_id)?;

				for layout_id in ids {
					result = result.intersected_with_id(&layout_id, context)?
				}

				result.compress(context)?;
				Ok(result)
			}
			None => Ok(Self::never(causes)),
		}
	}

	pub fn new(id: WithCauses<Id, F>) -> Result<Self, Error<F>> {
		let (id, causes) = id.into_parts();
		Ok(Self {
			ids: BTreeMap::new(),
			desc: WithCauses::new(IntersectedLayoutDescription::Alias(id), causes),
		})
	}

	pub fn from_parts(
		ids: BTreeMap<Id, Causes<F>>,
		desc: WithCauses<IntersectedLayoutDescription<F>, F>,
	) -> Self {
		Self { ids, desc }
	}

	pub fn never(causes: Causes<F>) -> Self {
		Self {
			ids: BTreeMap::new(),
			desc: WithCauses::new(IntersectedLayoutDescription::Never, causes),
		}
	}

	pub fn is_never(&self) -> bool {
		self.desc.is_never()
	}

	pub fn compress(&mut self, context: &Context<F, Descriptions>) -> Result<(), Error<F>> {
		if self.ids.is_empty() {
			let (desc, causes) = self.desc.parts_mut();
			self.ids = desc.compress(context, causes)?;
		}

		Ok(())
	}

	pub fn has_id(&self) -> bool {
		!self.ids.is_empty()
	}

	pub fn id(&self) -> Option<(Id, &Causes<F>)> {
		self.ids.iter().next().map(|(id, causes)| (*id, causes))
	}

	pub fn shared_id(&self, other: &Self) -> Option<WithCauses<Id, F>> {
		for (id, causes) in &self.ids {
			if let Some(other_causes) = other.ids.get(id) {
				return Some(WithCauses::new(
					*id,
					causes.clone().with(other_causes.iter().cloned()),
				));
			}
		}

		None
	}

	pub fn intersected_with_id(
		self,
		other: &WithCauses<Id, F>,
		context: &Context<F, Descriptions>,
	) -> Result<Self, Error<F>> {
		let other = Self::new(other.clone())?;
		self.intersected_with(other, context)
	}

	pub fn intersected_with(
		mut self,
		mut other: Self,
		context: &Context<F, Descriptions>,
	) -> Result<Self, Error<F>> {
		self.compress(context)?;
		other.compress(context)?;

		match self.shared_id(&other) {
			Some(id) => {
				let (id, causes) = id.into_parts();
				let mut ids = BTreeMap::new();
				ids.insert(id, causes.clone());

				Ok(Self {
					ids,
					desc: WithCauses::new(IntersectedLayoutDescription::Alias(id), causes),
				})
			}
			None => {
				let (desc, causes) = self.desc.into_parts();
				let other_causes = other.desc.causes().clone();

				Ok(Self {
					ids: BTreeMap::new(),
					desc: WithCauses::new(
						desc.intersected_with(
							causes.clone(),
							self.ids,
							other.desc,
							other.ids,
							context,
						)?,
						causes.with(other_causes),
					),
				})
			}
		}
	}

	pub fn apply_restrictions(
		self,
		restricted_fields: Vec<Loc<LayoutRestrictedField<F>, F>>,
	) -> Result<Self, Error<F>> {
		if restricted_fields.is_empty() {
			Ok(self)
		} else {
			Ok(Self {
				ids: BTreeMap::new(),
				desc: self
					.desc
					.try_map(|desc| desc.apply_restrictions(restricted_fields))?,
			})
		}
	}

	pub fn description(&self) -> &IntersectedLayoutDescription<F> {
		&self.desc
	}

	pub fn into_description(self) -> WithCauses<IntersectedLayoutDescription<F>, F> {
		self.desc
	}

	pub fn into_standard_description(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
		match self.ids.into_iter().next() {
			Some((id, _)) => Ok(treeldr_build::layout::Description::Alias(id)),
			None => self
				.desc
				.into_inner()
				.into_standard_description(source, target, vocabulary),
		}
	}

	pub fn into_layout(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> Result<WithCauses<Id, F>, Error<F>> {
		match self.ids.into_iter().next() {
			Some((id, causes)) => Ok(WithCauses::new(id, causes)),
			None => {
				let (desc, causes) = self.desc.into_parts();
				let standard_desc = desc.into_standard_description(source, target, vocabulary)?;

				let id = Id::Blank(vocabulary.new_blank_label());
				target.declare_layout(id, causes.preferred().cloned());
				target
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.replace_description(MaybeSet::new(standard_desc, causes.clone()));

				Ok(WithCauses::new(id, causes))
			}
		}
	}
}

impl<F: Clone + Ord> IntersectedLayoutDescription<F> {
	pub fn new(
		desc: Option<&WithCauses<LayoutDescription<F>, F>>,
		context: &Context<F, Descriptions>,
	) -> Result<Self, Error<F>> {
		match desc {
			None => Ok(Self::Never),
			Some(desc) => match desc.inner() {
				LayoutDescription::Standard(standard_desc) => match standard_desc {
					treeldr_build::layout::Description::Never => Ok(Self::Never),
					treeldr_build::layout::Description::Primitive(n) => Ok(Self::Primitive(*n)),
					treeldr_build::layout::Description::Literal(l) => Ok(Self::Literal(l.clone())),
					treeldr_build::layout::Description::Reference(r) => Ok(Self::Reference(*r)),
					treeldr_build::layout::Description::Struct(fields_id) => Ok(Self::Struct(
						IntersectedStruct::new(*fields_id, context, desc.causes())?,
					)),
					treeldr_build::layout::Description::Enum(variants_id) => Ok(Self::Enum(
						IntersectedEnum::new(*variants_id, context, desc.causes())?,
					)),
					treeldr_build::layout::Description::Set(s) => Ok(Self::Set(*s)),
					treeldr_build::layout::Description::Array(a) => Ok(Self::Array(a.clone())),
					treeldr_build::layout::Description::Alias(a) => Ok(Self::Alias(*a)),
				},
				LayoutDescription::Intersection(layouts_id, restricted_fields) => {
					let layout_list =
						context.require_list(*layouts_id, desc.causes().preferred().cloned())?;
					let mut layouts = Vec::new();
					for obj in layout_list.iter(context) {
						let obj = obj?;
						layouts.push(WithCauses::new(
							obj.as_id(obj.causes().preferred())?,
							obj.causes().clone(),
						))
					}

					let mut result =
						IntersectedLayout::try_from_iter(layouts, context, desc.causes().clone())?;
					result = result.apply_restrictions(restricted_fields.clone())?;
					Ok(result.into_description().into_inner())
				}
			},
		}
	}

	pub fn is_never(&self) -> bool {
		matches!(self, Self::Never)
	}

	pub fn compress(
		&mut self,
		context: &Context<F, Descriptions>,
		causes: &Causes<F>,
	) -> Result<BTreeMap<Id, Causes<F>>, Error<F>> {
		let mut aliases = BTreeMap::new();

		let mut causes = causes;

		while let Self::Alias(id) = self {
			if aliases.insert(*id, causes.clone()).is_none() {
				let layout = context.require_layout(*id, causes.preferred().cloned())?;
				causes = layout.causes();
				*self = Self::new(layout.description(), context)?;
			} else {
				break;
			}
		}

		Ok(aliases)
	}

	pub fn intersected_with(
		self,
		causes: Causes<F>,
		ids: BTreeMap<Id, Causes<F>>,
		other: WithCauses<IntersectedLayoutDescription<F>, F>,
		other_ids: BTreeMap<Id, Causes<F>>,
		context: &Context<F, Descriptions>,
	) -> Result<Self, Error<F>> {
		let (other, other_causes) = other.into_parts();
		match other {
			Self::Enum(b) => match self {
				Self::Enum(a) => a.intersected_with(b, context),
				non_enum => b.intersected_with_non_enum(
					IntersectedLayout::from_parts(ids, WithCauses::new(non_enum, causes)),
					context,
				),
			},
			other => match self {
				Self::Never => Ok(Self::Never),
				Self::Primitive(a) => match other {
					Self::Primitive(b) if a == b => Ok(Self::Primitive(a)),
					_ => Ok(Self::Never),
				},
				Self::Reference(a) => match other {
					Self::Reference(b) if a == b => Ok(Self::Reference(a)),
					_ => Ok(Self::Never),
				},
				Self::Literal(a) => match other {
					Self::Literal(b) if a == b => Ok(Self::Literal(a)),
					_ => Ok(Self::Never),
				},
				Self::Struct(a) => match other {
					Self::Struct(b) => a.intersected_with(b),
					_ => Ok(Self::Never),
				},
				Self::Enum(a) => a.intersected_with_non_enum(
					IntersectedLayout::from_parts(other_ids, WithCauses::new(other, other_causes)),
					context,
				),
				Self::Set(a) => match other {
					Self::Set(b) if a == b => Ok(Self::Set(a)),
					_ => Ok(Self::Never),
				},
				Self::Array(a) => match other {
					Self::Array(b) if a == b => Ok(Self::Array(a)),
					_ => Ok(Self::Never),
				},
				Self::Alias(a) => match other {
					Self::Alias(b) if a == b => Ok(Self::Alias(a)),
					_ => Ok(Self::Never),
				},
			},
		}
	}

	// pub fn intersected_with_id(self, other: &WithCauses<Id, F>, context: &Context<F, Descriptions>) -> Result<Self, Error<F>> {
	// 	let other = IntersectedLayout::new(other.clone(), context)?;
	// 	self.intersected_with(other.into_description())
	// }

	pub fn apply_restrictions(
		self,
		restricted_fields: Vec<Loc<LayoutRestrictedField<F>, F>>,
	) -> Result<Self, Error<F>> {
		match self {
			Self::Struct(mut s) => {
				if s.apply_restrictions(restricted_fields)? {
					Ok(Self::Struct(s))
				} else {
					Ok(Self::Never)
				}
			}
			other => {
				if let Some(Loc(_, loc)) = restricted_fields.get(0) {
					return Err(Loc(LocalError::UnexpectedFieldRestriction, loc.clone()).into());
				}

				Ok(other)
			}
		}
	}

	pub fn into_standard_description(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
		match self {
			Self::Never => Ok(treeldr_build::layout::Description::Never),
			Self::Primitive(n) => Ok(treeldr_build::layout::Description::Primitive(n)),
			Self::Reference(r) => Ok(treeldr_build::layout::Description::Reference(r)),
			Self::Literal(literal) => Ok(treeldr_build::layout::Description::Literal(literal)),
			Self::Struct(s) => s.into_standard_description(source, target, vocabulary),
			Self::Enum(e) => e.into_standard_description(source, target, vocabulary),
			Self::Set(s) => Ok(treeldr_build::layout::Description::Set(s)),
			Self::Array(a) => Ok(treeldr_build::layout::Description::Array(a)),
			Self::Alias(a) => Ok(treeldr_build::layout::Description::Alias(a)),
		}
	}
}
