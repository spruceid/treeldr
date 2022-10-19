use super::{IntersectedLayout, IntersectedLayoutDescription};
use crate::build::{
	Descriptions, Error, LayoutDescription, LayoutFieldCardinalityRestriction,
	LayoutFieldRangeRestriction, LayoutFieldRestriction, LayoutRestrictedField, LocalError,
};
use locspan::{BorrowStripped, Meta};
use locspan_derive::StrippedPartialEq;
use std::collections::BTreeMap;
use treeldr::{metadata::Merge, vocab::*, Id, MetaOption, Name};
use treeldr_build::{Context, ObjectToId};

#[derive(Clone)]
pub struct IntersectedStruct<M> {
	fields: Vec<Meta<IntersectedField<M>, M>>,
}

impl<M: Clone> IntersectedStruct<M> {
	pub fn new(
		fields_id: Id,
		context: &Context<M, Descriptions>,
		causes: &M,
	) -> Result<Self, Error<M>>
	where
		M: Merge,
	{
		let mut fields = Vec::new();

		for field_obj in context.require_list(fields_id, causes)?.iter(context) {
			let field_obj = field_obj?;
			let field_id = field_obj.as_id(field_obj.metadata())?;
			let field = context.require_layout_field(field_id, field_obj.metadata())?;

			let layout = match field.layout() {
				Some(field_layout) => FieldLayout::new(context, field_layout)?,
				None => panic!("no container layout"),
			};

			fields.push(Meta(
				IntersectedField {
					name: field.name().cloned().into(),
					property: field.property().cloned().into(),
					layout,
				},
				field.metadata().clone(),
			))
		}

		Ok(Self { fields })
	}

	/// Find a field that can be mapped to the given `field`
	pub fn mappable_field(&self, field: &IntersectedField<M>) -> Option<&IntersectedField<M>> {
		match field.property.value() {
			Some(prop) => {
				for other_field in &self.fields {
					if other_field.property.value() == Some(prop) {
						return Some(other_field);
					}
				}

				for other_field in &self.fields {
					if !other_field.property.is_set()
						&& other_field.name.value() == field.name.value()
					{
						return Some(other_field);
					}
				}
			}
			None => {
				for other_field in &self.fields {
					if other_field.name.value() == field.name.value() {
						return Some(other_field);
					}
				}
			}
		}

		None
	}

	/// Checks if this structure is logically included in `other`, meaning that
	/// every instance of this structure can be represented as an instance of
	/// `other`.
	pub fn is_included_in(&self, other: &Self) -> bool {
		// Each field must appear in `other`, or be optional.
		'next_field: for field in &self.fields {
			if field.is_required() {
				match other.mappable_field(field) {
					Some(other_field) if field.layout.is_included_in(&other_field.layout) => {
						continue 'next_field
					}
					_ => return false,
				}
			}
		}

		// Each field in `other` must appear in this structure.
		'next_field: for other_field in &other.fields {
			match self.mappable_field(other_field) {
				Some(field) if field.layout.is_included_in(&other_field.layout) => {
					continue 'next_field
				}
				_ => return false,
			}
		}

		true
	}

	pub fn intersected_with(
		mut self,
		mut other: IntersectedStruct<M>,
	) -> Result<IntersectedLayoutDescription<M>, Error<M>>
	where
		M: Merge,
	{
		let mut fields = std::mem::take(&mut self.fields);
		fields.reverse();
		other.fields.reverse();

		'next_field: while !fields.is_empty() || !other.fields.is_empty() {
			match fields.pop() {
				Some(field) => {
					let Meta(field, causes) = field;
					while let Some(other_field) = other.fields.pop() {
						if field.matches(&other_field) {
							let Meta(other_field, other_causes) = other_field;
							match field.intersected_with(other_field)? {
								Some(intersected_field) => self.fields.push(Meta(
									intersected_field,
									causes.merged_with(other_causes),
								)),
								None => return Ok(IntersectedLayoutDescription::Never),
							}

							continue 'next_field;
						} else {
							for after_field in &fields {
								if after_field.matches(&other_field) {
									for j in 0..other.fields.len() {
										if field.matches(&other.fields[j]) {
											panic!("unaligned layouts")
										}
									}
								}
							}

							self.fields.push(other_field);
						}
					}

					self.fields.push(Meta(field, causes));
				}
				None => {
					self.fields.push(other.fields.pop().unwrap());
				}
			}
		}

		Ok(IntersectedLayoutDescription::Struct(self))
	}

	pub fn apply_restriction(
		&mut self,
		Meta(restricted_field, meta): Meta<LayoutRestrictedField<M>, M>,
	) -> Result<bool, Error<M>>
	where
		M: Merge,
	{
		let prop_id = restricted_field.field_prop.map(|Meta(id, _)| id);
		let name = restricted_field.field_name.as_ref().map(|n| n.value());

		for (i, field) in self.fields.iter().enumerate() {
			let property_match = prop_id
				.and_then(|a| field.property.value().map(|b| a == *b))
				.unwrap_or(false);
			let name_match = name
				.and_then(|a| field.name.value().map(|b| a == b))
				.unwrap_or(false);

			if property_match || name_match {
				return self.fields[i]
					.layout
					.apply_restriction(restricted_field.restriction);
			}
		}

		Err(Meta(LocalError::FieldRestrictionNoMatches, meta).into())
	}

	pub fn apply_restrictions(
		&mut self,
		restricted_fields: Vec<Meta<LayoutRestrictedField<M>, M>>,
	) -> Result<bool, Error<M>>
	where
		M: Merge,
	{
		for r in restricted_fields {
			if !self.apply_restriction(r)? {
				return Ok(false);
			}
		}

		Ok(true)
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
		let mut fields = Vec::new();
		for Meta(field, causes) in self.fields {
			match field.into_field(source, target, vocabulary, causes.clone())? {
				Some(field_id) => fields.push(Meta(field_id.into_term(), causes)),
				None => return Ok(treeldr_build::layout::Description::Never),
			}
		}

		let fields_id = target.create_list(vocabulary, fields)?;
		Ok(treeldr_build::layout::Description::Struct(fields_id))
	}
}

impl<M> PartialEq for IntersectedStruct<M> {
	fn eq(&self, other: &Self) -> bool {
		self.fields.len() == other.fields.len()
			&& self
				.fields
				.iter()
				.zip(&other.fields)
				.all(|(a, b)| a.value() == b.value())
	}
}

#[derive(Clone, StrippedPartialEq)]
#[stripped_ignore(M)]
pub struct FieldLayout<M> {
	/// Layout description.
	desc: Meta<FieldLayoutDescription<M>, M>,

	/// Restrictions.
	restrictions: FieldRestrictions<M>,
}

#[derive(Clone, StrippedPartialEq)]
#[stripped_ignore(M)]
pub enum FieldLayoutDescription<M> {
	Required,
	Option,
	Array(Option<treeldr_build::layout::array::Semantics<M>>),
	Set,
}

impl<M> FieldLayout<M> {
	pub fn new(
		context: &Context<M, Descriptions>,
		Meta(layout_id, meta): &Meta<Id, M>,
	) -> Result<Self, Error<M>>
	where
		M: Clone + Merge,
	{
		let layout = context.require_layout(*layout_id, meta)?;

		match layout.description() {
			Some(desc) => {
				let desc_causes = desc.metadata();
				let (desc, item_id) = match desc.value() {
					LayoutDescription::Standard(treeldr_build::layout::Description::Required(
						r,
					)) => (FieldLayoutDescription::Required, *r),
					LayoutDescription::Standard(treeldr_build::layout::Description::Option(r)) => {
						(FieldLayoutDescription::Option, *r)
					}
					LayoutDescription::Standard(treeldr_build::layout::Description::Set(r)) => {
						(FieldLayoutDescription::Set, *r)
					}
					LayoutDescription::Standard(treeldr_build::layout::Description::Array(a)) => (
						FieldLayoutDescription::Array(a.semantics().cloned()),
						a.item_layout(),
					),
					_ => panic!("field layout not a container"),
				};

				Ok(Self {
					desc: Meta(desc, desc_causes.clone()),
					restrictions: FieldRestrictions::from_field_layout(Meta(item_id, meta.clone())),
				})
			}
			None => panic!("no container description"),
		}
	}

	pub fn is_required(&self) -> bool {
		self.desc.is_required()
	}

	pub fn is_included_in(&self, other: &Self) -> bool {
		self.desc.is_included_in(&other.desc)
			&& self.restrictions.stripped() == other.restrictions.stripped()
	}

	pub fn apply_restriction(
		&mut self,
		Meta(restriction, meta): Meta<LayoutFieldRestriction, M>,
	) -> Result<bool, Error<M>>
	where
		M: Clone + Merge,
	{
		match restriction {
			LayoutFieldRestriction::Range(r) => self.restrictions.range.insert(Meta(r, meta))?,
			LayoutFieldRestriction::Cardinality(c) => {
				if !self.restrictions.cardinality.insert(c) {
					return Ok(false);
				}
			}
		}

		Ok(true)
	}

	pub fn intersected_with(self, other: Self) -> Option<Self>
	where
		M: Clone + Merge,
	{
		if self.stripped() == other.stripped() {
			Some(self)
		} else {
			let Meta(desc, mut desc_causes) = self.desc;
			let Meta(other_desc, other_desc_causes) = other.desc;
			let desc = desc.intersected_with(other_desc)?;
			desc_causes.merge_with(other_desc_causes);
			let restrictions = self.restrictions.intersected_with(other.restrictions)?;

			Some(Self {
				desc: Meta(desc, desc_causes),
				restrictions,
			})
		}
	}

	pub fn into_layout(
		self,
		causes: M,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut Vocabulary,
	) -> Result<Option<Meta<Id, M>>, Error<M>>
	where
		M: Clone + Merge,
	{
		match self
			.restrictions
			.into_layout(causes, source, target, vocabulary)?
		{
			Some(item_layout) => {
				let Meta(desc, desc_causes) = self.desc;
				Ok(Some(desc.into_layout(
					item_layout,
					desc_causes,
					target,
					vocabulary,
				)))
			}
			None => Ok(None),
		}
	}
}

impl<M> FieldLayoutDescription<M> {
	pub fn is_required(&self) -> bool {
		matches!(self, Self::Required)
	}

	pub fn is_included_in(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Required, Self::Required | Self::Option) => true,
			(Self::Option, Self::Option) => true,
			(Self::Set, Self::Set) => true,
			(Self::Array(a), Self::Array(b)) => a.stripped() == b.stripped(),
			_ => false,
		}
	}

	pub fn intersected_with(self, other: Self) -> Option<Self> {
		match (self, other) {
			(Self::Required, Self::Required)
			| (Self::Required, Self::Option)
			| (Self::Option, Self::Required) => Some(Self::Required),
			(Self::Option, Self::Option) => Some(Self::Option),
			(Self::Set, Self::Set) => Some(Self::Set),
			(Self::Array(a), Self::Array(b)) if a.stripped() == b.stripped() => {
				Some(Self::Array(a))
			}
			_ => None,
		}
	}

	pub fn into_layout(
		self,
		item_layout: Meta<Id, M>,
		causes: M,
		target: &mut Context<M>,
		vocabulary: &mut Vocabulary,
	) -> Meta<Id, M>
	where
		M: Clone + Merge,
	{
		let Meta(item_layout, item_causes) = item_layout;
		let id = Id::Blank(vocabulary.new_blank_label());
		target.declare_layout(id, causes.clone());
		let layout = target.get_mut(id).unwrap().as_layout_mut().unwrap();

		match self {
			Self::Required => {
				layout.set_required(item_layout, item_causes).ok();
			}
			Self::Option => {
				layout.set_option(item_layout, item_causes).ok();
			}
			Self::Set => {
				layout.set_set(item_layout, item_causes).ok();
			}
			Self::Array(semantics) => {
				layout.set_array(item_layout, semantics, item_causes).ok();
			}
		}

		Meta(id, causes)
	}
}

#[derive(Clone)]
pub struct IntersectedField<M> {
	name: MetaOption<Name, M>,
	property: MetaOption<Id, M>,
	layout: FieldLayout<M>,
}

impl<M: Clone> IntersectedField<M> {
	pub fn is_required(&self) -> bool {
		self.layout.is_required()
	}

	pub fn into_field(
		self,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut Vocabulary,
		causes: M,
	) -> Result<Option<Id>, Error<M>>
	where
		M: Merge,
	{
		let cause = causes.clone();
		Ok(self
			.layout
			.into_layout(causes, source, target, vocabulary)?
			.map(|layout| {
				let id = Id::Blank(vocabulary.new_blank_label());
				target.declare_layout_field(id, cause);

				let def = target.get_mut(id).unwrap().as_layout_field_mut().unwrap();

				def.replace_name(self.name);
				def.replace_property(self.property);
				def.replace_layout(layout.into());

				id
			}))
	}

	pub fn matches(&self, other: &Self) -> bool {
		match (self.name.value(), other.name.value()) {
			(Some(a), Some(b)) => a == b,
			_ => match (self.property.value(), other.property.value()) {
				(Some(a), Some(b)) => a == b,
				_ => false,
			},
		}
	}

	pub fn intersected_with(self, other: Self) -> Result<Option<Self>, Error<M>>
	where
		M: Merge,
	{
		let name = match (self.name.unwrap(), other.name.unwrap()) {
			(Some(Meta(a, causes)), Some(b)) => {
				MetaOption::new(a, causes.merged_with(b.into_metadata()))
			}
			(Some(a), _) => a.into(),
			(_, Some(b)) => b.into(),
			(None, None) => MetaOption::default(),
		};

		let property = match (self.property.unwrap(), other.property.unwrap()) {
			(Some(Meta(a, causes)), Some(b)) => {
				MetaOption::new(a, causes.merged_with(b.into_metadata()))
			}
			(Some(a), _) => a.into(),
			(_, Some(b)) => b.into(),
			(None, None) => MetaOption::default(),
		};

		match self.layout.intersected_with(other.layout) {
			Some(layout) => Ok(Some(Self {
				name,
				property,
				layout,
			})),
			None => Ok(None),
		}
	}
}

impl<M> PartialEq for IntersectedField<M> {
	fn eq(&self, other: &Self) -> bool {
		self.name.value() == other.name.value()
			&& match (self.property.value(), other.property.value()) {
				(Some(a), Some(b)) => a == b && self.layout.stripped() == other.layout.stripped(),
				_ => false,
			}
	}
}

#[derive(Clone, StrippedPartialEq)]
#[stripped_ignore(M)]
pub struct FieldRestrictions<M> {
	range: RangeRestrictions<M>,

	#[stripped]
	cardinality: CardinalityRestrictions,
}

impl<M> Default for FieldRestrictions<M> {
	fn default() -> Self {
		Self {
			range: RangeRestrictions::default(),
			cardinality: CardinalityRestrictions::default(),
		}
	}
}

impl<M> FieldRestrictions<M> {
	pub fn from_field_layout(layout: Meta<Id, M>) -> Self {
		Self {
			range: RangeRestrictions::from_field_layout(layout),
			cardinality: CardinalityRestrictions::default(),
		}
	}

	pub fn intersected_with(self, other: Self) -> Option<Self>
	where
		M: Clone + Merge,
	{
		Some(Self {
			range: self.range.intersected_with(other.range)?,
			cardinality: self.cardinality.intersected_with(other.cardinality)?,
		})
	}

	pub fn into_layout(
		self,
		causes: M,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut Vocabulary,
	) -> Result<Option<Meta<Id, M>>, Error<M>>
	where
		M: Clone + Merge,
	{
		if !self.range.any.is_empty() {
			todo!("any range restriction")
		}

		if self.cardinality.min().is_some() || self.cardinality.max().is_some() {
			todo!("cardinality restriction")
		}

		let result = IntersectedLayout::try_from_iter(
			self.range
				.all
				.iter()
				.map(|(id, causes)| Meta(*id, causes.clone())),
			source,
			causes.clone(),
		)?;

		if result.has_id() || !result.needs_id() {
			Ok(Some(result.into_layout(source, target, vocabulary)?))
		} else {
			Err(Meta(
				LocalError::AnonymousFieldLayoutIntersection(
					self.range
						.all
						.into_iter()
						.map(|(id, causes)| Meta(id, causes))
						.collect(),
				),
				causes,
			)
			.into())
		}
	}
}

impl<M> PartialEq for FieldRestrictions<M> {
	fn eq(&self, other: &Self) -> bool {
		self.range.stripped() == other.range.stripped() && self.cardinality == other.cardinality
	}
}

#[derive(Clone)]
pub struct RangeRestrictions<M> {
	any: BTreeMap<Id, M>,
	all: BTreeMap<Id, M>,
}

impl<M> Default for RangeRestrictions<M> {
	fn default() -> Self {
		Self {
			any: BTreeMap::new(),
			all: BTreeMap::new(),
		}
	}
}

impl<M> RangeRestrictions<M> {
	pub fn from_field_layout(Meta(layout_id, meta): Meta<Id, M>) -> Self {
		let mut result = Self::default();

		result.all.insert(layout_id, meta);

		result
	}
}

impl<M> RangeRestrictions<M> {
	pub fn insert(
		&mut self,
		Meta(restriction, meta): Meta<LayoutFieldRangeRestriction, M>,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		match restriction {
			LayoutFieldRangeRestriction::Any(id) => {
				meta.merge_into_btree_map_entry(self.any.entry(id));

				Ok(())
			}
			LayoutFieldRangeRestriction::All(id) => {
				meta.merge_into_btree_map_entry(self.all.entry(id));
				Ok(())
			}
		}
	}

	pub fn causes(&self) -> Option<M>
	where
		M: Clone + Merge,
	{
		let mut result: Option<M> = None;

		for m in self.any.values() {
			match &mut result {
				Some(r) => r.merge_with(m.clone()),
				None => result = Some(m.clone()),
			}
		}

		for m in self.all.values() {
			match &mut result {
				Some(r) => r.merge_with(m.clone()),
				None => result = Some(m.clone()),
			}
		}

		result
	}

	pub fn intersected_with(self, mut other: Self) -> Option<Self>
	where
		M: Clone + Merge,
	{
		let mut all = BTreeMap::new();
		for (id, causes) in self.all {
			if let Some(other_causes) = other.all.remove(&id) {
				all.insert(id, causes.merged_with(other_causes));
			}
		}

		let mut any = BTreeMap::new();
		for (id, causes) in self.any {
			let causes = match other.any.remove(&id) {
				Some(other_causes) => causes.merged_with(other_causes),
				None => causes,
			};

			any.insert(id, causes);
		}
		any.extend(other.any);

		if all.is_empty() {
			None
		} else {
			Some(Self { all, any })
		}
	}
}

impl<M> locspan::StrippedPartialEq for RangeRestrictions<M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.any.len() == other.any.len()
			&& self.all.len() == other.all.len()
			&& self.any.keys().zip(other.any.keys()).all(|(a, b)| a == b)
			&& self.all.keys().zip(other.all.keys()).all(|(a, b)| a == b)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct CardinalityRestrictions {
	min: u32,
	max: u32,
}

impl Default for CardinalityRestrictions {
	fn default() -> Self {
		Self {
			min: 0,
			max: u32::MAX,
		}
	}
}

impl CardinalityRestrictions {
	pub fn min(&self) -> Option<u32> {
		if self.min > 0 {
			Some(self.min)
		} else {
			None
		}
	}

	pub fn max(&self) -> Option<u32> {
		if self.max < u32::MAX {
			Some(self.max)
		} else {
			None
		}
	}

	pub fn insert(&mut self, restriction: LayoutFieldCardinalityRestriction) -> bool {
		match restriction {
			LayoutFieldCardinalityRestriction::AtLeast(min) => {
				if min > self.max {
					return false;
				}

				self.min = std::cmp::max(min, self.min);

				true
			}
			LayoutFieldCardinalityRestriction::AtMost(max) => {
				if self.min > max {
					return false;
				}

				self.max = std::cmp::min(max, self.max);

				true
			}
			LayoutFieldCardinalityRestriction::Exactly(m) => {
				if self.min > m {
					return false;
				}

				if m > self.max {
					return false;
				}

				self.min = m;
				self.max = m;

				true
			}
		}
	}

	pub fn intersected_with(self, other: Self) -> Option<Self> {
		let min = std::cmp::max(self.min, other.min);
		let max = std::cmp::min(self.max, other.max);

		if max >= min {
			Some(Self { min, max })
		} else {
			None
		}
	}
}
