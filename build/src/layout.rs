use std::cmp::Ordering;

use crate::{
	context::{MapIds, MapIdsIn},
	error, rdf,
	resource::{self, BindingValueRef},
	functional_property_value::{self, FunctionalPropertyValue},
	utils::TryCollect,
	Context, Error, ObjectAsId, ObjectAsRequiredId,
};
use derivative::Derivative;
use locspan::Meta;
use rdf_types::IriVocabulary;
pub use treeldr::layout::{DescriptionProperty, Property};
use treeldr::{metadata::Merge, vocab::Object, Id, IriIndex, Multiple, Name, PropertyValueRef};

pub mod array;
pub mod field;
pub mod intersection;
pub mod primitive;
pub mod restriction;
pub mod variant;

pub use primitive::Primitive;
pub use restriction::{Restriction, Restrictions};

use primitive::BuildPrimitive;

#[derive(Debug)]
pub enum SingleDescriptionProperty<M> {
	DerivedFrom(FunctionalPropertyValue<Id, M>),
	Struct(FunctionalPropertyValue<Id, M>),
	Reference(FunctionalPropertyValue<Id, M>),
	Enum(FunctionalPropertyValue<Id, M>),
	Required(FunctionalPropertyValue<Id, M>),
	Option(FunctionalPropertyValue<Id, M>),
	Set(FunctionalPropertyValue<Id, M>),
	OneOrMany(FunctionalPropertyValue<Id, M>),
	Array(FunctionalPropertyValue<Id, M>),
	Alias(FunctionalPropertyValue<Id, M>)
}

#[derive(Debug, Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub enum SingleDescriptionPropertyRef<'a, M> {
	DerivedFrom(&'a FunctionalPropertyValue<Id, M>),
	Struct(&'a FunctionalPropertyValue<Id, M>),
	Reference(&'a FunctionalPropertyValue<Id, M>),
	Enum(&'a FunctionalPropertyValue<Id, M>),
	Required(&'a FunctionalPropertyValue<Id, M>),
	Option(&'a FunctionalPropertyValue<Id, M>),
	Set(&'a FunctionalPropertyValue<Id, M>),
	OneOrMany(&'a FunctionalPropertyValue<Id, M>),
	Array(&'a FunctionalPropertyValue<Id, M>),
	Alias(&'a FunctionalPropertyValue<Id, M>)
}

impl<'a, M> SingleDescriptionPropertyRef<'a, M> {
	pub fn metadata(&self) -> &M {
		match self {
			Self::DerivedFrom(p) => p.first().unwrap().value.into_metadata(),
			Self::Struct(p) => p.first().unwrap().value.into_metadata(),
			Self::Reference(p) => p.first().unwrap().value.into_metadata(),
			Self::Enum(p) => p.first().unwrap().value.into_metadata(),
			Self::Required(p) => p.first().unwrap().value.into_metadata(),
			Self::Option(p) => p.first().unwrap().value.into_metadata(),
			Self::Set(p) => p.first().unwrap().value.into_metadata(),
			Self::OneOrMany(p) => p.first().unwrap().value.into_metadata(),
			Self::Array(p) => p.first().unwrap().value.into_metadata(),
			Self::Alias(p) => p.first().unwrap().value.into_metadata()
		}
	}

	pub fn cloned(&self) -> SingleDescriptionProperty<M> where M: Clone {
		match *self {
			Self::DerivedFrom(p) => SingleDescriptionProperty::DerivedFrom(p.clone()),
			Self::Struct(p) => SingleDescriptionProperty::Struct(p.clone()),
			Self::Reference(p) => SingleDescriptionProperty::Reference(p.clone()),
			Self::Enum(p) => SingleDescriptionProperty::Enum(p.clone()),
			Self::Required(p) => SingleDescriptionProperty::Required(p.clone()),
			Self::Option(p) => SingleDescriptionProperty::Option(p.clone()),
			Self::Set(p) => SingleDescriptionProperty::Set(p.clone()),
			Self::OneOrMany(p) => SingleDescriptionProperty::OneOrMany(p.clone()),
			Self::Array(p) => SingleDescriptionProperty::Array(p.clone()),
			Self::Alias(p) => SingleDescriptionProperty::Alias(p.clone())
		}
	}

	pub fn build(
		&self,
		id: Id,
		restrictions: Restrictions<M>,
	) -> Result<treeldr::layout::Description<M>, Error<M>> where M: Clone + Merge {
		match *self {
			Self::DerivedFrom(p) => {
				let value = p.try_unwraped().map_err(|e| e.at_functional_node_property(id, DescriptionProperty::DerivedFrom))?;
				let derived = value.into_required().unwrap().try_map_borrow_metadata(|id, meta| {
					match Primitive::from_id(*id) {
						Some(p) => {
							p.build(id, restrictions.into_primitive(), meta)
						}
						None => {
							Err(Meta(error::LayoutNotPrimitive(*id).into(), meta.first().unwrap().value.into_metadata().clone()))
						}
					}
				});
				Ok(treeldr::layout::Description::Derived(p))
			},
			Self::Struct(p) => SingleDescriptionProperty::Struct(p.clone()),
			Self::Reference(p) => SingleDescriptionProperty::Reference(p.clone()),
			Self::Enum(p) => SingleDescriptionProperty::Enum(p.clone()),
			Self::Required(p) => SingleDescriptionProperty::Required(p.clone()),
			Self::Option(p) => SingleDescriptionProperty::Option(p.clone()),
			Self::Set(p) => SingleDescriptionProperty::Set(p.clone()),
			Self::OneOrMany(p) => SingleDescriptionProperty::OneOrMany(p.clone()),
			Self::Array(p) => SingleDescriptionProperty::Array(p.clone()),
			Self::Alias(p) => SingleDescriptionProperty::Alias(p.clone())
		}
	}
}

pub struct NoSingleDescription<'a, M> {
	a: SingleDescriptionPropertyRef<'a, M>,
	b: SingleDescriptionPropertyRef<'a, M>
}

/// Layout description properties.
#[derive(Debug, Clone, Derivative)]
#[derivative(Default(bound=""))]
pub struct DescriptionProperties<M> {
	derived_from: FunctionalPropertyValue<Id, M>,
	struct_: FunctionalPropertyValue<Id, M>,
	reference: FunctionalPropertyValue<Id, M>,
	enum_: FunctionalPropertyValue<Id, M>,
	required: FunctionalPropertyValue<Id, M>,
	option: FunctionalPropertyValue<Id, M>,
	set: FunctionalPropertyValue<Id, M>,
	one_or_many: FunctionalPropertyValue<Id, M>,
	array: FunctionalPropertyValue<Id, M>,
	alias: FunctionalPropertyValue<Id, M>
}

impl<M> DescriptionProperties<M> {
	pub fn set(
		&mut self,
		prop_cmp: impl Fn(Id, Id) -> Option<Ordering>,
		prop: DescriptionProperty,
		value: Meta<Object<M>, M>
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			DescriptionProperty::Alias => self
				.alias
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::Array => self
				.array
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::DerivedFrom => self
				.derived_from
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::Fields => self
				.struct_
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::OneOrMany => self
				.one_or_many
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::Option => self
				.option
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::Reference => self
				.reference
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::Required => self
				.required
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::Set => self
				.set
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			DescriptionProperty::Variants => self
				.enum_
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
		}

		Ok(())
	}

	pub fn is_included_in(&self, context: &Context<M>, other: &Self) -> bool {
		fn is_struct_included<M>(context: &Context<M>, a: Id, b: Id) -> bool {
			// all fields in `b` must include a field of `a`.
			match (context.get_list(a), context.get_list(b)) {
				(Some(a), Some(b)) => b.lenient_iter(context).all(|Meta(b, _)| {
					b.as_id()
						.map(|b| {
							a.lenient_iter(context).any(|Meta(a, _)| {
								a.as_id()
									.map(|a| field::is_included_in(context, a, b))
									.unwrap_or(false)
							})
						})
						.unwrap_or(false)
				}),
				_ => false,
			}
		}

		fn is_enum_included<M>(context: &Context<M>, a: Id, b: Id) -> bool {
			// all variants in `a` must be included in a variant in `b`.
			match (context.get_list(a), context.get_list(b)) {
				(Some(a), Some(b)) => a.lenient_iter(context).all(|Meta(a, _)| {
					a.as_id()
						.map(|a| {
							b.lenient_iter(context).any(|Meta(b, _)| {
								b.as_id()
									.map(|b| variant::is_included_in(context, a, b))
									.unwrap_or(false)
							})
						})
						.unwrap_or(false)
				}),
				_ => false,
			}
		}

		fn is_eq<M>(_context: &Context<M>, a: Id, b: Id) -> bool {
			a == b
		}

		fn check<M, const N: usize>(
			context: &Context<M>,
			is_included: impl Fn(&Context<M>, Id, Id) -> bool,
			a: &FunctionalPropertyValue<Id, M>,
			bs: [&FunctionalPropertyValue<Id, M>; N]
		) -> bool {
			if a.is_empty() ^ bs.iter().all(|b| b.is_empty()) {
				for b in bs {
					for a in a {
						for b in b {
							if !is_included(context, **a.value, **b.value) {
								return false;
							}
						}
					}
				}
	
				true
			} else {
				false
			}
		}

		check(context, is_eq, &self.alias, [&other.alias]) &&
		check(context, is_included_in, &self.reference, [&other.reference]) &&
		check(context, is_included_in, &self.required, [&other.required, &other.option, &other.one_or_many]) &&
		check(context, is_included_in, &self.option, [&other.option, &other.one_or_many]) &&
		check(context, is_included_in, &self.one_or_many, [&other.one_or_many]) &&
		check(context, is_included_in, &self.array, [&other.array]) &&
		check(context, is_included_in, &self.set, [&other.set]) &&
		check(context, is_struct_included, &self.struct_, [&other.struct_]) &&
		check(context, is_enum_included, &self.enum_, [&other.enum_])
	}

	pub fn collect_sub_layouts(
		&self,
		context: &Context<M>,
		sub_layouts: &mut Vec<SubLayout<M>>
	) where
		M: Clone,
	{
		for PropertyValueRef { value: Meta(fields_id, _), .. } in &self.struct_ {
			if let Some(fields) = context.get_list(*fields_id) {
				for Meta(object, _) in fields.lenient_iter(context) {
					if let Some(field_id) = object.as_id() {
						if let Some(field) = context
							.get(field_id)
							.map(resource::Definition::as_formatted)
						{
							for field_layout_id in field.format() {
								if let Some(field_layout) = context
									.get(**field_layout_id.value)
									.map(resource::Definition::as_layout)
								{
									sub_layouts.push(SubLayout {
										layout: field_layout_id.value.cloned(),
										connection: LayoutConnection::FieldContainer(field_id),
									});

									let field_desc = field_layout.description();
									
									for id in &field_desc.required {
										sub_layouts.push(SubLayout {
											layout: id.value.cloned(),
											connection: LayoutConnection::FieldItem(field_id),
										})
									}

									for id in &field_desc.option {
										sub_layouts.push(SubLayout {
											layout: id.value.cloned(),
											connection: LayoutConnection::FieldItem(field_id),
										})
									}

									for id in &field_desc.set {
										sub_layouts.push(SubLayout {
											layout: id.value.cloned(),
											connection: LayoutConnection::FieldItem(field_id),
										})
									}

									for id in &field_desc.one_or_many {
										sub_layouts.push(SubLayout {
											layout: id.value.cloned(),
											connection: LayoutConnection::FieldItem(field_id),
										})
									}

									for id in &field_desc.array {
										sub_layouts.push(SubLayout {
											layout: id.value.cloned(),
											connection: LayoutConnection::FieldItem(field_id),
										})
									}
								}
							}
						}
					}
				}
			}
		}

		for PropertyValueRef { value, .. } in &self.set {
			sub_layouts.push(SubLayout {
				layout: value.cloned(),
				connection: LayoutConnection::Item,
			})
		}

		for PropertyValueRef { value, .. } in &self.one_or_many {
			sub_layouts.push(SubLayout {
				layout: value.cloned(),
				connection: LayoutConnection::Item,
			})
		}

		for PropertyValueRef { value, .. } in &self.array {
			sub_layouts.push(SubLayout {
				layout: value.cloned(),
				connection: LayoutConnection::Item,
			})
		}
	}

	pub fn single_description(&self) -> Result<Option<SingleDescriptionPropertyRef<M>>, NoSingleDescription<M>> {
		let mut result = None;

		if !self.derived_from.is_empty() {
			let a = SingleDescriptionPropertyRef::DerivedFrom(&self.derived_from);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.struct_.is_empty() {
			let a = SingleDescriptionPropertyRef::Struct(&self.struct_);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.reference.is_empty() {
			let a = SingleDescriptionPropertyRef::Reference(&self.reference);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.enum_.is_empty() {
			let a = SingleDescriptionPropertyRef::Enum(&self.enum_);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.required.is_empty() {
			let a = SingleDescriptionPropertyRef::Required(&self.required);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.option.is_empty() {
			let a = SingleDescriptionPropertyRef::Option(&self.option);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.one_or_many.is_empty() {
			let a = SingleDescriptionPropertyRef::OneOrMany(&self.one_or_many);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.array.is_empty() {
			let a = SingleDescriptionPropertyRef::Array(&self.array);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.set.is_empty() {
			let a = SingleDescriptionPropertyRef::Set(&self.set);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		if !self.alias.is_empty() {
			let a = SingleDescriptionPropertyRef::Alias(&self.alias);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a)
			}
		}

		Ok(result)
	}

	pub fn build(&self, id: Id) -> Result<treeldr::layout::Description<M>, Error<M>> where M: Clone {
		match self.single_description() {
			Ok(Some(desc)) => desc.build(),
			Ok(None) => {
				match Primitive::from_id(id) {
					Some(p) => Ok(treeldr::layout::Description::Primitive(p)),
					None => Ok(treeldr::layout::Description::Never)
				}
			}
			Err(NoSingleDescription { a, b }) => Err(Meta(error::LayoutDescriptionMismatch {
				id,
				desc1: a.cloned(),
				desc2: b.cloned()
			}.into(), a.metadata().clone()))
		}
	}

	pub fn iter(&self)-> DescriptionPropertiesIter<M> {
		DescriptionPropertiesIter { derived_from: self.derived_from.iter(), struct_: self.struct_.iter(), reference: self.reference.iter(), enum_: self.enum_.iter(), required: self.required.iter(), option: self.option.iter(), set: self.set.iter(), one_or_many: self.one_or_many.iter(), array: self.array.iter(), alias: self.alias.iter() }
	}
}

pub struct DescriptionPropertiesIter<'a, M> {
	derived_from: functional_property_value::Iter<'a, Id, M>,
	struct_: functional_property_value::Iter<'a, Id, M>,
	reference: functional_property_value::Iter<'a, Id, M>,
	enum_: functional_property_value::Iter<'a, Id, M>,
	required: functional_property_value::Iter<'a, Id, M>,
	option: functional_property_value::Iter<'a, Id, M>,
	set: functional_property_value::Iter<'a, Id, M>,
	one_or_many: functional_property_value::Iter<'a, Id, M>,
	array: functional_property_value::Iter<'a, Id, M>,
	alias: functional_property_value::Iter<'a, Id, M>
}

impl<'a, M> Iterator for DescriptionPropertiesIter<'a, M> {
	type Item = Meta<DescriptionBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.derived_from
			.next()
			.map(|p| p.into_cloned_class_binding(DescriptionBinding::DerivedFrom))
			.or_else(|| self.struct_
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::Struct))
			)
			.or_else(|| self.reference
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::Reference))
			)
			.or_else(|| self.enum_
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::Enum))
			)
			.or_else(|| self.required
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::Required))
			)
			.or_else(|| self.option
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::Option))
			)
			.or_else(|| self.set
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::Set))
			)
			.or_else(|| self.one_or_many
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::OneOrMany))
			)
			.or_else(|| self.array
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::Array))
			)
			.or_else(|| self.alias
				.next()
				.map(|p| p.into_cloned_class_binding(DescriptionBinding::Alias))
			)
	}
}

impl<M> MapIds for DescriptionProperties<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.struct_.map_ids_in(Some(DescriptionProperty::Fields.into()), f);
		self.reference.map_ids_in(Some(DescriptionProperty::Reference.into()), f);
		self.enum_.map_ids_in(Some(DescriptionProperty::Variants.into()), f);
		self.required.map_ids_in(Some(DescriptionProperty::Required.into()), f);
		self.option.map_ids_in(Some(DescriptionProperty::Option.into()), f);
		self.set.map_ids_in(Some(DescriptionProperty::Set.into()), f);
		self.one_or_many.map_ids_in(Some(DescriptionProperty::OneOrMany.into()), f);
		self.array.map_ids_in(Some(DescriptionProperty::Array.into()), f);
		self.alias.map_ids_in(Some(DescriptionProperty::Alias.into()), f);
	}
}

pub enum DescriptionBinding {
	DerivedFrom(Option<Id>, Id),
	Struct(Option<Id>, Id),
	Reference(Option<Id>, Id),
	Enum(Option<Id>, Id),
	Required(Option<Id>, Id),
	Option(Option<Id>, Id),
	Set(Option<Id>, Id),
	OneOrMany(Option<Id>, Id),
	Array(Option<Id>, Id),
	Alias(Option<Id>, Id),
}

impl DescriptionBinding {
	pub fn property(&self) -> DescriptionProperty {
		match self {
			Self::DerivedFrom(_, _) => DescriptionProperty::DerivedFrom,
			Self::Reference(_, _) => DescriptionProperty::Reference,
			Self::Struct(_, _) => DescriptionProperty::Fields,
			Self::Enum(_, _) => DescriptionProperty::Variants,
			Self::Required(_, _) => DescriptionProperty::Required,
			Self::Option(_, _) => DescriptionProperty::Option,
			Self::Set(_, _) => DescriptionProperty::Set,
			Self::OneOrMany(_, _) => DescriptionProperty::OneOrMany,
			Self::Array(_, _) => DescriptionProperty::Array,
			Self::Alias(_, _) => DescriptionProperty::Alias,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::DerivedFrom(_, v) => BindingValueRef::Id(*v),
			Self::Reference(_, v) => BindingValueRef::Id(*v),
			Self::Struct(_, v) => BindingValueRef::Id(*v),
			Self::Enum(_, v) => BindingValueRef::Id(*v),
			Self::Required(_, v) => BindingValueRef::Id(*v),
			Self::Option(_, v) => BindingValueRef::Id(*v),
			Self::Set(_, v) => BindingValueRef::Id(*v),
			Self::OneOrMany(_, v) => BindingValueRef::Id(*v),
			Self::Array(_, v) => BindingValueRef::Id(*v),
			Self::Alias(_, v) => BindingValueRef::Id(*v),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
	Never,
	Primitive(Primitive),
	Reference,
	Literal,
	Struct,
	Enum,
	Required,
	Option,
	Set,
	OneOrMany,
	Array,
	Alias,
}

/// Layout definition.
#[derive(Clone)]
pub struct Definition<M> {
	/// Type for which this layout is defined.
	ty: FunctionalPropertyValue<Id, M>,

	/// Layout description.
	desc: DescriptionProperties<M>,

	intersection_of: FunctionalPropertyValue<Id, M>,

	/// List of restrictions.
	restrictions: FunctionalPropertyValue<Id, M>,

	/// List semantics.
	array_semantics: array::Semantics<M>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutConnection {
	FieldContainer(Id),
	FieldItem(Id),
	Variant(Id),
	Item,
}

pub struct SubLayout<M> {
	pub connection: LayoutConnection,
	pub layout: Meta<Id, M>,
}

pub struct ParentLayout {
	pub connection: LayoutConnection,
	pub layout: Id,
}

impl<M> Default for Definition<M> {
	fn default() -> Self {
		Self {
			ty: FunctionalPropertyValue::default(),
			desc: DescriptionProperties::default(),
			intersection_of: FunctionalPropertyValue::default(),
			restrictions: FunctionalPropertyValue::default(),
			array_semantics: array::Semantics::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.ty
	}

	pub fn ty_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.ty
	}

	pub fn description(&self) -> &DescriptionProperties<M> {
		&self.desc
	}

	pub fn description_mut(&mut self) -> &mut DescriptionProperties<M> {
		&mut self.desc
	}

	pub fn intersection_of(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.intersection_of
	}

	pub fn intersection_of_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.intersection_of
	}

	pub fn restrictions(&self) -> &FunctionalPropertyValue<Id, M> {
		&self.restrictions
	}

	pub fn restrictions_mut(&mut self) -> &mut FunctionalPropertyValue<Id, M> {
		&mut self.restrictions
	}

	pub fn is_included_in(&self, context: &Context<M>, other: &Self) -> bool {
		self.desc.is_included_in(context, &other.desc)
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(Id, Id) -> Option<Ordering>,
		prop: Property,
		value: Meta<Object<M>, M>
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::For => self.ty_mut().insert(None, prop_cmp, rdf::from::expect_id(value)?),
			Property::Description(prop) => self.desc.set(prop_cmp, prop, value)?,
			Property::WithRestrictions => {
				self.restrictions_mut().insert(None, prop_cmp, rdf::from::expect_id(value)?)
			}
			Property::IntersectionOf => self
				.intersection_of_mut()
				.insert(None, prop_cmp, rdf::from::expect_id(value)?),
			Property::ArrayListFirst => {
				self.array_semantics.set_first(rdf::from::expect_id(value)?)
			}
			Property::ArrayListRest => self.array_semantics.set_rest(rdf::from::expect_id(value)?),
			Property::ArrayListNil => self.array_semantics.set_nil(rdf::from::expect_id(value)?),
		}

		Ok(())
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			ty: self.ty.iter(),
			desc: self.desc.iter(),
			intersection_of: self.intersection_of.iter(),
			restrictions: self.restrictions.iter(),
			array_semantics: self.array_semantics.bindings(),
		}
	}
}

impl<M: Merge> Definition<M> {
	pub fn set_derived_from(&mut self, primitive: Meta<Primitive, M>) {
		self.desc.derived_from.insert_base(primitive.cast())
	}

	pub fn set_alias(&mut self, alias: Meta<Id, M>) {
		self.desc.alias.insert_base(alias)
	}

	pub fn set_fields(&mut self, fields: Meta<Id, M>) {
		self.desc.struct_.insert_base(fields)
	}

	pub fn set_reference(&mut self, ty: Meta<Id, M>) {
		self.desc.reference.insert_base(ty)
	}

	pub fn set_enum(&mut self, variants: Meta<Id, M>) {
		self.desc.enum_.insert_base(variants)
	}

	pub fn set_required(&mut self, item: Meta<Id, M>) {
		self.desc.required.insert_base(item)
	}

	pub fn set_option(&mut self, item: Meta<Id, M>) {
		self.desc.option.insert_base(item)
	}

	pub fn set_set(&mut self, item: Meta<Id, M>) {
		self.desc.set.insert_base(item)
	}

	pub fn set_one_or_many(&mut self, item: Meta<Id, M>) {
		self.desc.one_or_many.insert_base(item)
	}

	pub fn set_array(&mut self, item: Meta<Id, M>) {
		self.desc.array.insert_base(item)
	}

	pub fn set_array_list_first(&mut self, first_prop: Meta<Id, M>) {
		self.array_semantics.set_first(first_prop)
	}

	pub fn set_array_list_rest(&mut self, value: Meta<Id, M>) {
		self.array_semantics.set_rest(value)
	}

	pub fn set_array_list_nil(&mut self, value: Meta<Id, M>) {
		self.array_semantics.set_nil(value)
	}

	pub fn set_array_semantics(&mut self, value: array::Semantics<M>) {
		self.array_semantics.unify_with(value)
	}
}

impl<M: Clone> Definition<M> {
	pub fn sub_layouts(&self, context: &Context<M>) -> Vec<SubLayout<M>> {
		let mut sub_layouts = Vec::new();

		self.desc.collect_sub_layouts(context, &mut sub_layouts);

		sub_layouts
	}

	/// Build a default name for this layout.
	pub fn default_name(
		&self,
		context: &Context<M>,
		vocabulary: &impl IriVocabulary<Iri = IriIndex>,
		parent_layouts: &[Meta<ParentLayout, M>],
		as_resource: &resource::Data<M>,
	) -> Option<Meta<Name, M>> {
		if let Id::Iri(term) = as_resource.id {
			if let Ok(Some(name)) = Name::from_iri(vocabulary.iri(&term).unwrap()) {
				return Some(Meta(name, as_resource.metadata.clone()));
			}
		}

		// if let Some(Description::Literal(regexp)) = self.desc.value() {
		// 	if let Some(functional_property_valueton) = regexp.as_functional_property_valueton() {
		// 		if let Ok(functional_property_valueton_name) = Name::new(functional_property_valueton) {
		// 			let mut name = Name::new("const").unwrap();
		// 			name.push_name(&functional_property_valueton_name);
		// 			return Ok(Some(Meta(name, metadata)));
		// 		}
		// 	}
		// }

		if parent_layouts.len() == 1 {
			let parent = &parent_layouts[0];
			let parent_layout = context.get(parent.layout).unwrap().as_component();

			if let Some(parent_layout_name) = parent_layout.name().first() {
				match parent.connection {
					LayoutConnection::FieldItem(field_id) => {
						let field = context.get(field_id).unwrap().as_component();

						if let Some(field_name) = field.name().first() {
							let mut name = parent_layout_name.value.into_value().clone();
							name.push_name(&field_name.value.value());

							return Some(Meta(name, as_resource.metadata.clone()));
						}
					}
					LayoutConnection::Item => {
						let mut name = parent_layout_name.value.into_value().clone();
						name.push_name(&Name::new("item").unwrap());

						return Some(Meta(name, as_resource.metadata.clone()));
					}
					_ => (),
				}
			}
		}

		None
	}

	/// Generates the intersection definition for this layout.
	///
	/// If `None` is returned, this means the intersection cannot be defined yet,
	/// because it depends on the not-yet-computed intersection of other layouts.
	pub fn intersection_definition(
		&self,
		context: &crate::Context<M>,
		as_resource: &resource::Data<M>,
	) -> Result<Option<intersection::Definition<M>>, Error<M>>
	where
		M: Merge,
	{
		let mut result =
			intersection::Definition::new(Meta(as_resource.id, as_resource.metadata.clone()));

		#[derive(Debug, Clone, Copy)]
		struct Incomplete;

		for PropertyValueRef { value: Meta(list_id, meta), .. } in &self.intersection_of {
			let list = context.require_list(*list_id).map_err(|e| {
				e.at_node_property(as_resource.id, Property::IntersectionOf, meta.clone())
			})?;

			let intersections = list.try_fold(
				context,
				Ok(None),
				|intersection: Result<Option<intersection::Definition<M>>, Incomplete>, items| {
					match intersection {
						Ok(intersection) => {
							let mut result = Vec::new();

							for PropertyValueRef { value: Meta(object, layout_metadata), .. } in items {
								let layout_id = object.as_required_id(layout_metadata)?;

								let new_intersection = match intersection::Definition::from_id(
									context,
									Meta(layout_id, layout_metadata.clone()),
								)? {
									Some(desc) => Some(match &intersection {
										Some(intersection) => {
											let mut new_intersection = intersection.clone();
											new_intersection.intersect_with(desc);
											new_intersection
										}
										None => desc,
									}),
									None => return Ok(vec![Err(Incomplete)]),
								};

								result.push(Ok(new_intersection))
							}

							Ok(result)
						}
						Err(Incomplete) => Ok(vec![Err(Incomplete)]),
					}
				},
			);

			for intersection in intersections {
				match intersection? {
					Err(Incomplete) => return Ok(None),
					Ok(Some(def)) => result.add(def),
					Ok(None) => result.add_never(meta.clone()),
				}
			}
		}

		Ok(Some(result))
	}

	pub fn add(&mut self, def: intersection::BuiltDefinition<M>)
	where
		M: Merge,
	{
		self.desc.extend(def.desc)
	}

	pub fn build_description(
		&self,
		context: &crate::Context<M>,
		as_resource: &treeldr::node::Data<M>,
		_as_component: &treeldr::component::Data<M>,
		metadata: &M,
	) -> Result<Meta<treeldr::layout::Description<M>, M>, Error<M>>
	where
		M: Merge,
	{
		let restrictions_id = self.restrictions.clone().try_unwrap().map_err(|e| {
			e.at_functional_node_property(as_resource.id, Property::WithRestrictions)
		})?;

		let restrictions = restrictions_id
			.as_required()
			.map(|restrictions_id| {
				let list_id = restrictions_id.value();
				let mut restrictions = Restrictions::default();
				let list = context.require_list(*list_id).map_err(|e| {
					e.at_node_property(as_resource.id, Property::WithRestrictions, restrictions_id.sub_property_metadata().clone())
				})?;
				for restriction_value in list.iter(context) {
					let Meta(restriction_value, meta) = restriction_value?;
					let restriction_id = restriction_value.as_required_id(meta)?;
					let restriction_definition = context
						.require(restriction_id)
						.map_err(|e| e.at(meta.clone()))?
						.require_layout_restriction(context)
						.map_err(|e| e.at(meta.clone()))?;
					let restriction = restriction_definition.build()?;
					restrictions.insert(restriction)?
				}

				Ok(restrictions)
			})
			.transpose()?
			.unwrap_or_default();

		let desc =
			self.desc.build();

		match desc.unwrap() {
			Some(Meta(desc, desc_metadata)) => {
				let desc = match desc {
					Description::Never => treeldr::layout::Description::Never,
					Description::Primitive(n) => treeldr::layout::Description::Primitive(n.build(
						as_resource.id,
						restrictions.into_primitive(),
						&desc_metadata,
					)?),
					Description::Reference(layout_id) => {
						let layout_ref = context.require_layout_id(layout_id).map_err(|e| {
							e.at_node_property(
								as_resource.id,
								DescriptionProperty::Reference,
								desc_metadata.clone(),
							)
						})?;
						let r = treeldr::layout::Reference::new(Meta(
							layout_ref,
							desc_metadata.clone(),
						));
						treeldr::layout::Description::Reference(r)
					}
					Description::Struct(fields_id) => {
						let fields = context
							.require_list(fields_id)
							.map_err(|e| {
								e.at_node_property(
									as_resource.id,
									DescriptionProperty::Fields,
									desc_metadata.clone(),
								)
							})?
							.iter(context)
							.map(|item| {
								let Meta(object, field_metadata) = item?.cloned();
								let field_id = object.into_required_id(&field_metadata)?;
								Ok(Meta(
									context
										.require_layout_field_id(field_id)
										.map_err(|e| e.at(field_metadata.clone()))?,
									field_metadata,
								))
							})
							.try_collect()?;

						let strct = treeldr::layout::Struct::new(fields);
						treeldr::layout::Description::Struct(strct)
					}
					Description::Enum(options_id) => {
						let variants: Vec<_> = context
							.require_list(options_id)
							.map_err(|e| {
								e.at_node_property(
									as_resource.id,
									DescriptionProperty::Variants,
									desc_metadata.clone(),
								)
							})?
							.iter(context)
							.map(|item| {
								let Meta(object, variant_metadata) = item?.cloned();
								let variant_id = object.into_required_id(&variant_metadata)?;
								Ok(Meta(
									context
										.require_layout_variant_id(variant_id)
										.map_err(|e| e.at(variant_metadata.clone()))?,
									variant_metadata,
								))
							})
							.try_collect()?;

						let enm = treeldr::layout::Enum::new(variants);
						treeldr::layout::Description::Enum(enm)
					}
					Description::Required(item_layout_id) => {
						let item_layout_ref =
							context.require_layout_id(item_layout_id).map_err(|e| {
								e.at_node_property(
									as_resource.id,
									DescriptionProperty::Required,
									desc_metadata.clone(),
								)
							})?;
						treeldr::layout::Description::Required(treeldr::layout::Required::new(
							Meta(item_layout_ref, desc_metadata.clone()),
						))
					}
					Description::Option(item_layout_id) => {
						let item_layout_ref =
							context.require_layout_id(item_layout_id).map_err(|e| {
								e.at_node_property(
									as_resource.id,
									DescriptionProperty::Option,
									desc_metadata.clone(),
								)
							})?;
						treeldr::layout::Description::Option(treeldr::layout::Optional::new(Meta(
							item_layout_ref,
							desc_metadata.clone(),
						)))
					}
					Description::Set(item_layout_id) => {
						let item_layout_ref =
							context.require_layout_id(item_layout_id).map_err(|e| {
								e.at_node_property(
									as_resource.id,
									DescriptionProperty::Set,
									desc_metadata.clone(),
								)
							})?;
						treeldr::layout::Description::Set(treeldr::layout::Set::new(
							Meta(item_layout_ref, desc_metadata.clone()),
							restrictions.into_container(),
						))
					}
					Description::OneOrMany(item_layout_id) => {
						let item_layout_ref =
							context.require_layout_id(item_layout_id).map_err(|e| {
								e.at_node_property(
									as_resource.id,
									DescriptionProperty::OneOrMany,
									desc_metadata.clone(),
								)
							})?;
						treeldr::layout::Description::OneOrMany(treeldr::layout::OneOrMany::new(
							Meta(item_layout_ref, desc_metadata.clone()),
							restrictions.into_container(),
						))
					}
					Description::Array(item_layout_id) => {
						let item_layout_ref =
							context.require_layout_id(item_layout_id).map_err(|e| {
								e.at_node_property(
									as_resource.id,
									DescriptionProperty::Array,
									desc_metadata.clone(),
								)
							})?;
						let semantics = self
							.array_semantics
							.clone()
							.build(context, as_resource.id)?;
						treeldr::layout::Description::Array(treeldr::layout::Array::new(
							Meta(item_layout_ref, desc_metadata.clone()),
							restrictions.into_container(),
							semantics,
						))
					}
					Description::Alias(alias_layout_id) => {
						let alias_layout_ref =
							context.require_layout_id(alias_layout_id).map_err(|e| {
								e.at_node_property(
									as_resource.id,
									DescriptionProperty::Alias,
									desc_metadata.clone(),
								)
							})?;
						treeldr::layout::Description::Alias(alias_layout_ref)
					}
				};

				Ok(Meta(desc, desc_metadata))
			}
			None => match Primitive::from_id(as_resource.id) {
				Some(p) => Ok(Meta(
					treeldr::layout::Description::Primitive(p.into()),
					metadata.clone(),
				)),
				None => Err(Meta(
					error::LayoutDescriptionMissing(as_resource.id).into(),
					metadata.clone(),
				)),
			},
		}
	}

	pub(crate) fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		as_component: &treeldr::component::Data<M>,
		metadata: M,
	) -> Result<Meta<treeldr::layout::Definition<M>, M>, Error<M>>
	where
		M: Merge,
	{
		let intersection_of = self
			.intersection_of
			.clone()
			.try_unwrap()
			.map_err(|e| e.at_functional_node_property(as_resource.id, Property::IntersectionOf))?
			.try_map_borrow_metadata(|id, meta| {
				let list = context.require_list(id).map_err(|e| {
					e.at_node_property(as_resource.id, Property::IntersectionOf, meta.clone())
				})?;
				let mut intersection = Multiple::default();
				for item in list.iter(context) {
					let Meta(object, layout_meta) = item?;
					let layout_id = object.as_required_id(layout_meta)?;
					let layout_tid = context
						.require_layout_id(layout_id)
						.map_err(|e| e.at(layout_meta.clone()))?;
					intersection.insert(Meta(layout_tid, layout_meta.clone()))
				}

				Ok(intersection)
			})?;

		let desc = self.build_description(context, as_resource, as_component, &metadata)?;
		let ty =
			self.ty
				.clone()
				.into_type_at_node_binding(context, as_resource.id, Property::For)?;

		Ok(Meta(
			treeldr::layout::Definition::new(ty, desc, intersection_of),
			metadata,
		))
	}
}

pub fn is_included_in<M>(context: &Context<M>, a: Id, b: Id) -> bool {
	if a == b {
		true
	} else {
		context
			.get(a)
			.and_then(|a| {
				context
					.get(b)
					.map(|b| a.as_layout().is_included_in(context, b.as_layout()))
			})
			.unwrap_or(false)
	}
}

impl<M: Merge> MapIds for Definition<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.ty.map_ids_in(Some(Property::For.into()), &f);
		self.desc.map_ids(&f);
		self.intersection_of
			.map_ids_in(Some(Property::IntersectionOf.into()), &f);
		self.restrictions
			.map_ids_in(Some(Property::WithRestrictions.into()), &f);
		self.array_semantics.map_ids(f)
	}
}

pub enum ClassBinding {
	For(Id),
	Description(DescriptionBinding),
	IntersectionOf(Id),
	WithRestrictions(Id),
	ArraySemantics(array::Binding),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::For(_) => Property::For,
			Self::Description(d) => Property::Description(d.property()),
			Self::IntersectionOf(_) => Property::IntersectionOf,
			Self::WithRestrictions(_) => Property::WithRestrictions,
			Self::ArraySemantics(b) => b.property(),
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::For(v) => BindingValueRef::Id(*v),
			Self::Description(d) => d.value(),
			Self::IntersectionOf(v) => BindingValueRef::Id(*v),
			Self::WithRestrictions(v) => BindingValueRef::Id(*v),
			Self::ArraySemantics(b) => b.value(),
		}
	}
}

pub struct ClassBindings<'a, M> {
	ty: functional_property_value::Iter<'a, Id, M>,
	desc: functional_property_value::Iter<'a, Description, M>,
	intersection_of: functional_property_value::Iter<'a, Id, M>,
	restrictions: functional_property_value::Iter<'a, Id, M>,
	array_semantics: array::Bindings<'a, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.ty
			.next()
			.map(|v| v.into_cloned_value().map(ClassBinding::For))
			.or_else(|| {
				self.desc
					.next()
					.and_then(|Meta(v, meta)| {
						v.into_binding()
							.map(|b| Meta(ClassBinding::Description(b), meta))
					})
					.or_else(|| {
						self.intersection_of
							.next()
							.map(|v| v.into_cloned_value().map(ClassBinding::IntersectionOf))
							.or_else(|| {
								self.restrictions
									.next()
									.map(|v| {
										v.into_cloned_value().map(ClassBinding::WithRestrictions)
									})
									.or_else(|| {
										self.array_semantics
											.next()
											.map(|v| v.map(ClassBinding::ArraySemantics))
									})
							})
					})
			})
	}
}
