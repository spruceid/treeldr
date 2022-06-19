use super::{IntersectedLayout, IntersectedLayoutDescription};
use crate::build::{
	Descriptions, Error, LayoutFieldCardinalityRestriction, LayoutFieldRangeRestriction,
	LayoutFieldRestriction, LayoutRestrictedField, LocalError,
};
use locspan::Loc;
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
			fields.push(WithCauses::new(
				IntersectedField {
					name: field.name().cloned().into(),
					property: field.property().cloned().into(),
					restrictions: FieldRestrictions::from_field_layout(field.layout().cloned()),
				},
				field.causes().clone(),
			))
		}

		Ok(Self { fields })
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
		Loc(restricted_field, loc): Loc<LayoutRestrictedField<F>, F>,
	) -> Result<bool, Error<F>> {
		let prop_id = restricted_field.field_prop.map(|Loc(id, _)| id);
		let name = restricted_field.field_name.as_ref().map(|n| n.value());

		for (i, field) in self.fields.iter().enumerate() {
			let property_match = prop_id
				.and_then(|a| field.property.value().map(|b| a == *b))
				.unwrap_or(false);
			let name_match = name
				.and_then(|a| field.name.value().map(|b| a == b))
				.unwrap_or(false);

			if property_match || name_match {
				match restricted_field.restriction {
					Loc(LayoutFieldRestriction::Range(r), loc) => {
						self.fields[i].restrictions.range.insert(Loc(r, loc))?
					}
					Loc(LayoutFieldRestriction::Cardinality(c), _) => {
						if !self.fields[i].restrictions.cardinality.insert(c) {
							return Ok(false);
						}
					}
				}

				return Ok(true);
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

#[derive(Clone)]
pub struct IntersectedField<F> {
	name: MaybeSet<Name, F>,
	property: MaybeSet<Id, F>,
	restrictions: FieldRestrictions<F>,
}

impl<F: Clone + Ord> IntersectedField<F> {
	pub fn into_field(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
		causes: Causes<F>,
	) -> Result<Option<Id>, Error<F>> {
		let cause = causes.preferred().cloned();
		Ok(self
			.restrictions
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

		match self.restrictions.intersected_with(other.restrictions) {
			Some(restrictions) => Ok(Some(Self {
				name,
				property,
				restrictions,
			})),
			None => Ok(None),
		}
	}
}

impl<F> PartialEq for IntersectedField<F> {
	fn eq(&self, other: &Self) -> bool {
		self.name.value() == other.name.value()
			&& match (self.property.value(), other.property.value()) {
				(Some(a), Some(b)) => a == b && self.restrictions == other.restrictions,
				_ => false,
			}
	}
}

#[derive(Clone)]
pub struct FieldRestrictions<F> {
	range: RangeRestrictions<F>,
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
	pub fn from_field_layout(layout: Option<WithCauses<Id, F>>) -> Self {
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
				.into_iter()
				.map(|(id, causes)| WithCauses::new(id, causes)),
			source,
			causes,
		)?;

		if result.has_id() {
			Ok(Some(result.into_layout(source, target, vocabulary)?))
		} else {
			Ok(None)
		}
	}
}

impl<F> PartialEq for FieldRestrictions<F> {
	fn eq(&self, other: &Self) -> bool {
		self.range == other.range && self.cardinality == other.cardinality
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
	pub fn from_field_layout(layout: Option<WithCauses<Id, F>>) -> Self {
		let mut result = Self::default();

		if let Some(layout) = layout {
			let (id, causes) = layout.into_parts();
			result.all.insert(id, causes);
		}

		result
	}
}

impl<F> RangeRestrictions<F> {
	pub fn insert(
		&mut self,
		Loc(restriction, loc): Loc<LayoutFieldRangeRestriction, F>,
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

impl<F> PartialEq for RangeRestrictions<F> {
	fn eq(&self, other: &Self) -> bool {
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
