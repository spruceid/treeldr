use super::{IntersectedLayout, IntersectedLayoutDescription};
use crate::build::{
	Descriptions, Error, LayoutDescription, LayoutFieldCardinalityRestriction,
	LayoutFieldRangeRestriction, LayoutFieldRestriction, LayoutRestrictedField, LocalError,
};
use locspan::{BorrowStripped, Loc, Meta};
use locspan_derive::StrippedPartialEq;
use std::collections::BTreeMap;
use treeldr::{vocab::*, Caused, Causes, Id, MaybeSet, Name, WithCauses};
use treeldr_build::{Context, ObjectToId};

#[derive(Clone)]
pub struct IntersectedStruct<F> {
	fields: Vec<WithCauses<IntersectedField<F>, F>>,
}

impl<F: Clone + Ord> IntersectedStruct<F> {
	pub fn new(
		fields_id: Id,
		context: &Context<F, Descriptions>,
		causes: &Causes<F>,
	) -> Result<Self, Error<F>> {
		let mut fields = Vec::new();

		for field_obj in context
			.require_list(fields_id, causes.preferred().cloned())?
			.iter(context)
		{
			let field_obj = field_obj?;
			let field_id = field_obj.as_id(field_obj.causes().preferred())?;
			let field =
				context.require_layout_field(field_id, field_obj.causes().preferred().cloned())?;

			let layout = match field.layout() {
				Some(field_layout) => FieldLayout::new(context, field_layout)?,
				None => panic!("no container layout"),
			};

			fields.push(WithCauses::new(
				IntersectedField {
					name: field.name().cloned().into(),
					property: field.property().cloned().into(),
					layout,
				},
				field.causes().clone(),
			))
		}

		Ok(Self { fields })
	}

	/// Find a field that can be mapped to the given `field`
	pub fn mappable_field(&self, field: &IntersectedField<F>) -> Option<&IntersectedField<F>> {
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
		mut other: IntersectedStruct<F>,
	) -> Result<IntersectedLayoutDescription<F>, Error<F>> {
		let mut fields = std::mem::take(&mut self.fields);
		fields.reverse();
		other.fields.reverse();

		'next_field: while !fields.is_empty() || !other.fields.is_empty() {
			match fields.pop() {
				Some(field) => {
					let (field, causes) = field.into_parts();
					while let Some(other_field) = other.fields.pop() {
						if field.matches(&other_field) {
							let (other_field, other_causes) = other_field.into_parts();
							match field.intersected_with(other_field)? {
								Some(intersected_field) => self.fields.push(WithCauses::new(
									intersected_field,
									causes.with(other_causes),
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

					self.fields.push(WithCauses::new(field, causes));
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
		Meta(restricted_field, loc): Loc<LayoutRestrictedField<F>, F>,
	) -> Result<bool, Error<F>> {
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

		Err(Loc(LocalError::FieldRestrictionNoMatches, loc).into())
	}

	pub fn apply_restrictions(
		&mut self,
		restricted_fields: Vec<Loc<LayoutRestrictedField<F>, F>>,
	) -> Result<bool, Error<F>> {
		for r in restricted_fields {
			if !self.apply_restriction(r)? {
				return Ok(false);
			}
		}

		Ok(true)
	}

	pub fn into_standard_description(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
		let mut fields = Vec::new();
		for field in self.fields {
			let (field, causes) = field.into_parts();
			let cause = causes.preferred().cloned();
			match field.into_field(source, target, vocabulary, causes)? {
				Some(field_id) => fields.push(Caused::new(field_id.into_term(), cause)),
				None => return Ok(treeldr_build::layout::Description::Never),
			}
		}

		let fields_id = target.create_list(vocabulary, fields)?;
		Ok(treeldr_build::layout::Description::Struct(fields_id))
	}
}

impl<F> PartialEq for IntersectedStruct<F> {
	fn eq(&self, other: &Self) -> bool {
		self.fields.len() == other.fields.len()
			&& self
				.fields
				.iter()
				.zip(&other.fields)
				.all(|(a, b)| a.inner() == b.inner())
	}
}

#[derive(Clone, StrippedPartialEq)]
#[stripped_ignore(F)]
pub struct FieldLayout<F> {
	/// Layout description.
	desc: WithCauses<FieldLayoutDescription<F>, F>,

	/// Restrictions.
	restrictions: FieldRestrictions<F>,
}

#[derive(Clone, StrippedPartialEq)]
#[stripped_ignore(F)]
pub enum FieldLayoutDescription<F> {
	Required,
	Option,
	Array(Option<treeldr_build::layout::array::Semantics<F>>),
	Set,
}

impl<F> FieldLayout<F> {
	pub fn new(
		context: &Context<F, Descriptions>,
		layout_id: &WithCauses<Id, F>,
	) -> Result<Self, Error<F>>
	where
		F: Clone + Ord,
	{
		let (layout_id, causes) = layout_id.clone().into_parts();
		let layout = context.require_layout(layout_id, causes.preferred().cloned())?;

		match layout.description() {
			Some(desc) => {
				let desc_causes = desc.causes();
				let (desc, item_id) = match desc.inner() {
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
					desc: WithCauses::new(desc, desc_causes.clone()),
					restrictions: FieldRestrictions::from_field_layout(WithCauses::new(
						item_id, causes,
					)),
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
		Meta(restriction, loc): Loc<LayoutFieldRestriction, F>,
	) -> Result<bool, Error<F>>
	where
		F: Clone + Ord,
	{
		match restriction {
			LayoutFieldRestriction::Range(r) => self.restrictions.range.insert(Loc(r, loc))?,
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
		F: Clone + Ord,
	{
		if self.stripped() == other.stripped() {
			Some(self)
		} else {
			let (desc, desc_causes) = self.desc.into_parts();
			let (other_desc, other_desc_causes) = other.desc.into_parts();
			let desc = desc.intersected_with(other_desc)?;
			let desc_causes = desc_causes.with(other_desc_causes);
			let restrictions = self.restrictions.intersected_with(other.restrictions)?;

			Some(Self {
				desc: WithCauses::new(desc, desc_causes),
				restrictions,
			})
		}
	}

	pub fn into_layout(
		self,
		causes: Causes<F>,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> Result<Option<WithCauses<Id, F>>, Error<F>>
	where
		F: Clone + Ord,
	{
		match self
			.restrictions
			.into_layout(causes, source, target, vocabulary)?
		{
			Some(item_layout) => {
				let (desc, desc_causes) = self.desc.into_parts();
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

impl<F> FieldLayoutDescription<F> {
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
		item_layout: WithCauses<Id, F>,
		causes: Causes<F>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> WithCauses<Id, F>
	where
		F: Clone + Ord,
	{
		let (item_layout, item_causes) = item_layout.into_parts();
		let id = Id::Blank(vocabulary.new_blank_label());
		target.declare_layout(id, causes.preferred().cloned());
		let layout = target.get_mut(id).unwrap().as_layout_mut().unwrap();

		match self {
			Self::Required => {
				layout
					.set_required(item_layout, item_causes.preferred().cloned())
					.ok();
			}
			Self::Option => {
				layout
					.set_option(item_layout, item_causes.preferred().cloned())
					.ok();
			}
			Self::Set => {
				layout
					.set_set(item_layout, item_causes.preferred().cloned())
					.ok();
			}
			Self::Array(semantics) => {
				layout
					.set_array(item_layout, semantics, item_causes.preferred().cloned())
					.ok();
			}
		}

		WithCauses::new(id, causes)
	}
}

#[derive(Clone)]
pub struct IntersectedField<F> {
	name: MaybeSet<Name, F>,
	property: MaybeSet<Id, F>,
	layout: FieldLayout<F>,
}

impl<F: Clone + Ord> IntersectedField<F> {
	pub fn is_required(&self) -> bool {
		self.layout.is_required()
	}

	pub fn into_field(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
		causes: Causes<F>,
	) -> Result<Option<Id>, Error<F>> {
		let cause = causes.preferred().cloned();
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

	pub fn intersected_with(self, other: Self) -> Result<Option<Self>, Error<F>> {
		let name = match (self.name.unwrap(), other.name.unwrap()) {
			(Some(a), Some(b)) => {
				let (a, causes) = a.into_parts();
				MaybeSet::new(a, causes.with(b.into_causes()))
			}
			(Some(a), _) => a.into(),
			(_, Some(b)) => b.into(),
			(None, None) => MaybeSet::default(),
		};

		let property = match (self.property.unwrap(), other.property.unwrap()) {
			(Some(a), Some(b)) => {
				let (a, causes) = a.into_parts();
				MaybeSet::new(a, causes.with(b.into_causes()))
			}
			(Some(a), _) => a.into(),
			(_, Some(b)) => b.into(),
			(None, None) => MaybeSet::default(),
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

impl<F> PartialEq for IntersectedField<F> {
	fn eq(&self, other: &Self) -> bool {
		self.name.value() == other.name.value()
			&& match (self.property.value(), other.property.value()) {
				(Some(a), Some(b)) => a == b && self.layout.stripped() == other.layout.stripped(),
				_ => false,
			}
	}
}

#[derive(Clone, StrippedPartialEq)]
#[stripped_ignore(F)]
pub struct FieldRestrictions<F> {
	range: RangeRestrictions<F>,

	#[stripped]
	cardinality: CardinalityRestrictions,
}

impl<F> Default for FieldRestrictions<F> {
	fn default() -> Self {
		Self {
			range: RangeRestrictions::default(),
			cardinality: CardinalityRestrictions::default(),
		}
	}
}

impl<F> FieldRestrictions<F> {
	pub fn from_field_layout(layout: WithCauses<Id, F>) -> Self {
		Self {
			range: RangeRestrictions::from_field_layout(layout),
			cardinality: CardinalityRestrictions::default(),
		}
	}

	pub fn intersected_with(self, other: Self) -> Option<Self>
	where
		F: Clone + Ord,
	{
		Some(Self {
			range: self.range.intersected_with(other.range)?,
			cardinality: self.cardinality.intersected_with(other.cardinality)?,
		})
	}

	pub fn into_layout(
		self,
		causes: Causes<F>,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> Result<Option<WithCauses<Id, F>>, Error<F>>
	where
		F: Clone + Ord,
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
				.map(|(id, causes)| WithCauses::new(*id, causes.clone())),
			source,
			causes.clone(),
		)?;

		if result.has_id() || !result.needs_id() {
			Ok(Some(result.into_layout(source, target, vocabulary)?))
		} else {
			Err(Loc(
				LocalError::AnonymousFieldLayoutIntersection(
					self.range
						.all
						.into_iter()
						.map(|(id, causes)| WithCauses::new(id, causes))
						.collect(),
				),
				causes.preferred().cloned().unwrap(),
			)
			.into())
		}
	}
}

impl<F> PartialEq for FieldRestrictions<F> {
	fn eq(&self, other: &Self) -> bool {
		self.range.stripped() == other.range.stripped() && self.cardinality == other.cardinality
	}
}

#[derive(Clone)]
pub struct RangeRestrictions<F> {
	any: BTreeMap<Id, Causes<F>>,
	all: BTreeMap<Id, Causes<F>>,
}

impl<F> Default for RangeRestrictions<F> {
	fn default() -> Self {
		Self {
			any: BTreeMap::new(),
			all: BTreeMap::new(),
		}
	}
}

impl<F> RangeRestrictions<F> {
	pub fn from_field_layout(layout: WithCauses<Id, F>) -> Self {
		let mut result = Self::default();

		let (id, causes) = layout.into_parts();
		result.all.insert(id, causes);

		result
	}
}

impl<F> RangeRestrictions<F> {
	pub fn insert(
		&mut self,
		Meta(restriction, loc): Loc<LayoutFieldRangeRestriction, F>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		match restriction {
			LayoutFieldRangeRestriction::Any(id) => {
				self.any.entry(id).or_default().add(loc);
				Ok(())
			}
			LayoutFieldRangeRestriction::All(id) => {
				self.all.entry(id).or_default().add(loc);
				Ok(())
			}
		}
	}

	pub fn causes(&self) -> Causes<F>
	where
		F: Clone + Ord,
	{
		let mut all_causes = Causes::new();

		for causes in self.any.values() {
			all_causes.extend(causes.iter().cloned())
		}

		for causes in self.all.values() {
			all_causes.extend(causes.iter().cloned())
		}

		all_causes
	}

	pub fn intersected_with(self, mut other: Self) -> Option<Self>
	where
		F: Clone + Ord,
	{
		let mut all = BTreeMap::new();
		for (id, causes) in self.all {
			if let Some(other_causes) = other.all.remove(&id) {
				all.insert(id, causes.with(other_causes));
			}
		}

		let mut any = BTreeMap::new();
		for (id, causes) in self.any {
			let causes = match other.any.remove(&id) {
				Some(other_causes) => causes.with(other_causes),
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

impl<F> locspan::StrippedPartialEq for RangeRestrictions<F> {
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
