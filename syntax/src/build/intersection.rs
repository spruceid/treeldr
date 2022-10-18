use super::{Descriptions, Error, LayoutDescription, LayoutRestrictedField, LocalError};
use derivative::Derivative;
use locspan::Meta;
use std::collections::BTreeMap;
use treeldr::{metadata::Merge, Id, MetaOption, Vocabulary};
use treeldr_build::{
	layout::{Array, RestrictedPrimitive},
	Context, ObjectToId,
};

mod enumeration;
mod structure;

pub use enumeration::*;
pub use structure::*;

#[derive(Clone)]
pub struct IntersectedLayout<M> {
	ids: BTreeMap<Id, M>,
	desc: Meta<IntersectedLayoutDescription<M>, M>,
}

impl<M> PartialEq for IntersectedLayout<M> {
	fn eq(&self, other: &Self) -> bool {
		self.ids.keys().any(|id| other.ids.contains_key(id))
			|| (self.ids.is_empty()
				&& other.ids.is_empty()
				&& self.desc.value() == other.desc.value())
	}
}

#[derive(Clone, Derivative)]
#[derivative(PartialEq(bound = ""))]
pub enum IntersectedLayoutDescription<M> {
	Never,
	Primitive(RestrictedPrimitive<M>),
	Struct(IntersectedStruct<M>),
	Reference(Id),
	Enum(IntersectedEnum<M>),
	Required(Id),
	Option(Id),
	Set(Id),
	Array(Array<M>),
	Alias(Id),
}

impl<M: Clone> IntersectedLayout<M> {
	pub fn try_from_iter<I: IntoIterator<Item = Meta<Id, M>>>(
		ids: I,
		context: &Context<M, Descriptions>,
		causes: M,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
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

	pub fn new(Meta(id, meta): Meta<Id, M>) -> Result<Self, Error<M>> {
		Ok(Self {
			ids: BTreeMap::new(),
			desc: Meta::new(IntersectedLayoutDescription::Alias(id), meta),
		})
	}

	pub fn from_parts(
		ids: BTreeMap<Id, M>,
		desc: Meta<IntersectedLayoutDescription<M>, M>,
	) -> Self {
		Self { ids, desc }
	}

	pub fn never(causes: M) -> Self {
		Self {
			ids: BTreeMap::new(),
			desc: Meta::new(IntersectedLayoutDescription::Never, causes),
		}
	}

	pub fn is_never(&self) -> bool {
		self.desc.is_never()
	}

	pub fn compress(&mut self, context: &Context<M, Descriptions>) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		if self.ids.is_empty() {
			let Meta(desc, causes) = &mut self.desc;
			self.ids = desc.compress(context, causes)?;
		}

		Ok(())
	}

	pub fn has_id(&self) -> bool {
		!self.ids.is_empty()
	}

	pub fn needs_id(&self) -> bool {
		self.desc.needs_id()
	}

	pub fn id(&self) -> Option<(Id, &M)> {
		self.ids.iter().next().map(|(id, causes)| (*id, causes))
	}

	pub fn shared_id(&self, other: &Self) -> Option<Meta<Id, M>>
	where
		M: Merge,
	{
		for (id, causes) in &self.ids {
			if let Some(other_causes) = other.ids.get(id) {
				return Some(Meta::new(
					*id,
					causes.clone().merged_with(other_causes.clone()),
				));
			}
		}

		None
	}

	pub fn into_required_id(self) -> Result<Meta<Id, M>, Error<M>> {
		match self.ids.into_iter().next() {
			Some((id, causes)) => Ok(Meta::new(id, causes)),
			None => todo!("anonymous intersection"),
		}
	}

	pub fn intersected_with_id(
		self,
		other: &Meta<Id, M>,
		context: &Context<M, Descriptions>,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
		let other = Self::new(other.clone())?;
		self.intersected_with(other, context)
	}

	pub fn intersected_with(
		mut self,
		mut other: Self,
		context: &Context<M, Descriptions>,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
		self.compress(context)?;
		other.compress(context)?;

		match self.shared_id(&other) {
			Some(Meta(id, causes)) => {
				let mut ids = BTreeMap::new();
				ids.insert(id, causes.clone());

				Ok(Self {
					ids,
					desc: Meta::new(IntersectedLayoutDescription::Alias(id), causes),
				})
			}
			None => {
				if self.desc.is_included_in(&other.desc) {
					return Ok(self);
				}

				if other.desc.is_included_in(&self.desc) {
					return Ok(other);
				}

				let Meta(desc, causes) = self.desc;
				let other_causes = other.desc.metadata().clone();

				Ok(Self {
					ids: BTreeMap::new(),
					desc: Meta::new(
						desc.intersected_with(
							causes.clone(),
							self.ids,
							other.desc,
							other.ids,
							context,
						)?,
						causes.merged_with(other_causes),
					),
				})
			}
		}
	}

	pub fn apply_restrictions(
		self,
		restricted_fields: Vec<Meta<LayoutRestrictedField<M>, M>>,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
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

	pub fn description(&self) -> &IntersectedLayoutDescription<M> {
		&self.desc
	}

	pub fn into_description(self) -> Meta<IntersectedLayoutDescription<M>, M> {
		self.desc
	}

	pub fn into_standard_description(
		self,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut Vocabulary,
	) -> Result<treeldr_build::layout::Description<M>, Error<M>>
	where
		M: Merge,
	{
		match self.ids.into_iter().next() {
			Some((id, _)) => Ok(treeldr_build::layout::Description::Alias(id)),
			None => self
				.desc
				.into_value()
				.into_standard_description(source, target, vocabulary),
		}
	}

	pub fn into_layout(
		self,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut Vocabulary,
	) -> Result<Meta<Id, M>, Error<M>>
	where
		M: Merge,
	{
		match self.ids.into_iter().next() {
			Some((id, causes)) => Ok(Meta::new(id, causes)),
			None => {
				let Meta(desc, causes) = self.desc;
				let standard_desc = desc.into_standard_description(source, target, vocabulary)?;

				let id = Id::Blank(vocabulary.new_blank_label());
				target.declare_layout(id, causes.clone());
				target
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.replace_description(MetaOption::new(standard_desc, causes.clone()));

				Ok(Meta::new(id, causes))
			}
		}
	}
}

impl<M: Clone> IntersectedLayoutDescription<M> {
	pub fn new(
		desc: Option<&Meta<LayoutDescription<M>, M>>,
		context: &Context<M, Descriptions>,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
		match desc {
			None => Ok(Self::Never),
			Some(desc) => match desc.value() {
				LayoutDescription::Standard(standard_desc) => match standard_desc {
					treeldr_build::layout::Description::Never => Ok(Self::Never),
					treeldr_build::layout::Description::Primitive(n) => {
						Ok(Self::Primitive(n.clone()))
					}
					treeldr_build::layout::Description::Reference(r) => Ok(Self::Reference(*r)),
					treeldr_build::layout::Description::Struct(fields_id) => Ok(Self::Struct(
						IntersectedStruct::new(*fields_id, context, desc.metadata())?,
					)),
					treeldr_build::layout::Description::Enum(variants_id) => Ok(Self::Enum(
						IntersectedEnum::new(*variants_id, context, desc.metadata())?,
					)),
					treeldr_build::layout::Description::Required(r) => Ok(Self::Required(*r)),
					treeldr_build::layout::Description::Option(s) => Ok(Self::Option(*s)),
					treeldr_build::layout::Description::Set(s) => Ok(Self::Set(*s)),
					treeldr_build::layout::Description::Array(a) => Ok(Self::Array(a.clone())),
					treeldr_build::layout::Description::Alias(a) => Ok(Self::Alias(*a)),
				},
				LayoutDescription::Intersection(layouts_id, restricted_fields) => {
					let layout_list = context.require_list(*layouts_id, desc.metadata())?;
					let mut layouts = Vec::new();
					for obj in layout_list.iter(context) {
						let obj = obj?;
						layouts.push(Meta::new(
							obj.as_id(obj.metadata())?,
							obj.metadata().clone(),
						))
					}

					let mut result = IntersectedLayout::try_from_iter(
						layouts,
						context,
						desc.metadata().clone(),
					)?;
					result = result.apply_restrictions(restricted_fields.clone())?;
					Ok(result.into_description().into_value())
				}
			},
		}
	}

	pub fn is_never(&self) -> bool {
		matches!(self, Self::Never)
	}

	pub fn never_or_else(self, f: impl FnOnce(Self) -> Self) -> Self {
		if self.is_never() {
			self
		} else {
			f(self)
		}
	}

	pub fn compress(
		&mut self,
		context: &Context<M, Descriptions>,
		causes: &M,
	) -> Result<BTreeMap<Id, M>, Error<M>>
	where
		M: Merge,
	{
		let mut aliases = BTreeMap::new();

		while let Self::Alias(id) = self {
			if aliases.insert(*id, causes.clone()).is_none() {
				let layout = context.require_layout(*id, causes)?;
				*self = Self::new(layout.description(), context)?;
			} else {
				break;
			}
		}

		Ok(aliases)
	}

	pub fn is_included_in(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Never, Self::Never) => true,
			(Self::Primitive(a), Self::Primitive(b)) => a.is_included_in(b),
			(Self::Reference(a), Self::Reference(b)) => a == b,
			(Self::Enum(a), Self::Enum(b)) => a.is_included_in(b),
			(Self::Struct(a), Self::Struct(b)) => a.is_included_in(b),
			(Self::Set(a), Self::Set(b)) => a == b,
			(Self::Array(a), Self::Array(b)) => a == b,
			(Self::Alias(a), Self::Alias(b)) => a == b,
			_ => false,
		}
	}

	pub fn needs_id(&self) -> bool {
		matches!(self, Self::Enum(_) | Self::Struct(_) | Self::Alias(_))
	}

	pub fn intersected_with(
		self,
		causes: M,
		ids: BTreeMap<Id, M>,
		Meta(other, other_causes): Meta<IntersectedLayoutDescription<M>, M>,
		other_ids: BTreeMap<Id, M>,
		context: &Context<M, Descriptions>,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
		match other {
			Self::Enum(b) => match self {
				Self::Enum(a) => a.intersected_with(b, context),
				non_enum => b.intersected_with_non_enum(
					IntersectedLayout::from_parts(ids, Meta::new(non_enum, causes)),
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
				Self::Struct(a) => match other {
					Self::Struct(b) => a.intersected_with(b),
					_ => Ok(Self::Never),
				},
				Self::Enum(a) => a.intersected_with_non_enum(
					IntersectedLayout::from_parts(other_ids, Meta::new(other, other_causes)),
					context,
				),
				Self::Required(a) => match other {
					Self::Required(b) => {
						eprintln!("case B");
						let c = IntersectedLayout::try_from_iter(
							[Meta::new(a, causes.clone()), Meta::new(b, other_causes)],
							context,
							causes,
						)?
						.into_required_id()?
						.into_value();
						Ok(Self::Required(c))
					}
					_ => Ok(Self::Never),
				},
				Self::Option(a) => match other {
					Self::Option(b) => {
						let c = IntersectedLayout::try_from_iter(
							[Meta::new(a, causes.clone()), Meta::new(b, other_causes)],
							context,
							causes,
						)?
						.into_required_id()?
						.into_value();
						Ok(Self::Option(c))
					}
					_ => Ok(Self::Never),
				},
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

	pub fn apply_restrictions(
		self,
		restricted_fields: Vec<Meta<LayoutRestrictedField<M>, M>>,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
		match self {
			Self::Struct(mut s) => {
				if s.apply_restrictions(restricted_fields)? {
					Ok(Self::Struct(s))
				} else {
					Ok(Self::Never)
				}
			}
			other => {
				if let Some(Meta(_, loc)) = restricted_fields.get(0) {
					return Err(Meta(LocalError::UnexpectedFieldRestriction, loc.clone()).into());
				}

				Ok(other)
			}
		}
	}

	pub fn into_standard_description(
		self,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut Vocabulary,
	) -> Result<treeldr_build::layout::Description<M>, Error<M>>
	where
		M: Merge,
	{
		match self {
			Self::Never => Ok(treeldr_build::layout::Description::Never),
			Self::Primitive(n) => Ok(treeldr_build::layout::Description::Primitive(n)),
			Self::Reference(r) => Ok(treeldr_build::layout::Description::Reference(r)),
			Self::Struct(s) => s.into_standard_description(source, target, vocabulary),
			Self::Enum(e) => e.into_standard_description(source, target, vocabulary),
			Self::Required(r) => Ok(treeldr_build::layout::Description::Required(r)),
			Self::Option(o) => Ok(treeldr_build::layout::Description::Option(o)),
			Self::Set(s) => Ok(treeldr_build::layout::Description::Set(s)),
			Self::Array(a) => Ok(treeldr_build::layout::Description::Array(a)),
			Self::Alias(a) => Ok(treeldr_build::layout::Description::Alias(a)),
		}
	}
}
