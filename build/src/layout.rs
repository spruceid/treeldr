use std::cmp::Ordering;

use crate::{
	context::{MapIds, MapIdsIn},
	error,
	functional_property_value::{self, FunctionalPropertyValue},
	resource::{self, BindingValueRef},
	utils::TryCollect,
	Context, Error, MetaValueExt,
};
use derivative::Derivative;
use locspan::Meta;
use rdf_types::IriVocabulary;
pub use treeldr::layout::{DescriptionProperty, Property};
use treeldr::{
	metadata::Merge, prop::UnknownProperty, Id, IriIndex, MetaOption, Multiple, Name,
	PropertyValueRef, PropertyValues, TId, Value,
};

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
	Map(FunctionalPropertyValue<Id, M>),
	OneOrMany(FunctionalPropertyValue<Id, M>),
	Array(FunctionalPropertyValue<Id, M>),
	Alias(FunctionalPropertyValue<Id, M>),
}

#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum SingleDescriptionPropertyRef<'a, M> {
	DerivedFrom(&'a FunctionalPropertyValue<Id, M>),
	Struct(&'a FunctionalPropertyValue<Id, M>),
	Reference(&'a FunctionalPropertyValue<Id, M>),
	Enum(&'a FunctionalPropertyValue<Id, M>),
	Required(&'a FunctionalPropertyValue<Id, M>),
	Option(&'a FunctionalPropertyValue<Id, M>),
	Set(&'a FunctionalPropertyValue<Id, M>),
	Map(&'a FunctionalPropertyValue<Id, M>),
	OneOrMany(&'a FunctionalPropertyValue<Id, M>),
	Array(&'a FunctionalPropertyValue<Id, M>),
	Alias(&'a FunctionalPropertyValue<Id, M>),
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
			Self::Map(p) => p.first().unwrap().value.into_metadata(),
			Self::OneOrMany(p) => p.first().unwrap().value.into_metadata(),
			Self::Array(p) => p.first().unwrap().value.into_metadata(),
			Self::Alias(p) => p.first().unwrap().value.into_metadata(),
		}
	}

	pub fn cloned(&self) -> SingleDescriptionProperty<M>
	where
		M: Clone,
	{
		match *self {
			Self::DerivedFrom(p) => SingleDescriptionProperty::DerivedFrom(p.clone()),
			Self::Struct(p) => SingleDescriptionProperty::Struct(p.clone()),
			Self::Reference(p) => SingleDescriptionProperty::Reference(p.clone()),
			Self::Enum(p) => SingleDescriptionProperty::Enum(p.clone()),
			Self::Required(p) => SingleDescriptionProperty::Required(p.clone()),
			Self::Option(p) => SingleDescriptionProperty::Option(p.clone()),
			Self::Set(p) => SingleDescriptionProperty::Set(p.clone()),
			Self::Map(p) => SingleDescriptionProperty::Map(p.clone()),
			Self::OneOrMany(p) => SingleDescriptionProperty::OneOrMany(p.clone()),
			Self::Array(p) => SingleDescriptionProperty::Array(p.clone()),
			Self::Alias(p) => SingleDescriptionProperty::Alias(p.clone()),
		}
	}

	pub fn build(
		&self,
		context: &Context<M>,
		id: Id,
		restrictions: MetaOption<Restrictions<M>, M>,
		array_semantics: &array::Semantics<M>,
		map_value: FunctionalPropertyValue<Id, M>,
	) -> Result<treeldr::layout::Description<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		match *self {
			Self::DerivedFrom(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::DerivedFrom(None))
				})?;
				let derived =
					value
						.into_required()
						.unwrap()
						.try_map_borrow_metadata(|id, meta| match Primitive::from_id(id) {
							Some(p) => p.build(id, restrictions.map(Restrictions::into_primitive)),
							None => {
								let meta = meta.first().unwrap().value.into_metadata();
								Err(Meta(error::LayoutNotPrimitive(id).into(), meta.clone()))
							}
						})?;
				Ok(treeldr::layout::Description::Derived(derived))
			}
			Self::Struct(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Fields(None))
				})?;
				Ok(treeldr::layout::Description::Struct(
					value
						.into_required()
						.unwrap()
						.try_map_borrow_metadata(|fields_id, meta| {
							let fields = context
								.require_list(fields_id)
								.map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::Fields(None),
										meta.first().unwrap().value.into_metadata().clone(),
									)
								})?
								.iter(context)
								.map(|item| {
									let object: Meta<Value, M> = item?.cloned();
									let Meta(field_id, field_metadata) =
										object.into_expected_id()?;
									Ok(Meta(
										context
											.require_layout_field_id(field_id)
											.map_err(|e| e.at(field_metadata.clone()))?,
										field_metadata,
									))
								})
								.try_collect()?;

							Ok(treeldr::layout::Struct::new(fields))
						})?,
				))
			}
			Self::Enum(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Variants(None))
				})?;
				Ok(treeldr::layout::Description::Enum(
					value.into_required().unwrap().try_map_borrow_metadata(
						|options_id, meta| {
							let variants: Vec<_> = context
								.require_list(options_id)
								.map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::Variants(None),
										meta.first().unwrap().value.into_metadata().clone(),
									)
								})?
								.iter(context)
								.map(|item| {
									let object = item?.cloned();
									let Meta(variant_id, variant_metadata) =
										object.into_expected_id()?;
									Ok(Meta(
										context
											.require_layout_variant_id(variant_id)
											.map_err(|e| e.at(variant_metadata.clone()))?,
										variant_metadata,
									))
								})
								.try_collect()?;

							Ok(treeldr::layout::Enum::new(variants))
						},
					)?,
				))
			}
			Self::Reference(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Reference(None))
				})?;
				Ok(treeldr::layout::Description::Reference(
					value
						.into_required()
						.unwrap()
						.try_map_borrow_metadata(|layout_id, meta| {
							let layout_ref = context.require_layout_id(layout_id).map_err(|e| {
								e.at_node_property(
									id,
									DescriptionProperty::Reference(None),
									meta.first().unwrap().value.into_metadata().clone(),
								)
							})?;
							Ok(treeldr::layout::Reference::new(Meta(
								layout_ref,
								meta.first().unwrap().value.into_metadata().clone(),
							)))
						})?,
				))
			}
			Self::Required(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Required(None))
				})?;
				Ok(treeldr::layout::Description::Required(
					value.into_required().unwrap().try_map_borrow_metadata(
						|item_layout_id, meta| {
							let item_layout_ref =
								context.require_layout_id(item_layout_id).map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::Required(None),
										meta.first().unwrap().value.into_metadata().clone(),
									)
								})?;
							Ok(treeldr::layout::Required::new(Meta(
								item_layout_ref,
								meta.first().unwrap().value.into_metadata().clone(),
							)))
						},
					)?,
				))
			}
			Self::Option(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Option(None))
				})?;
				Ok(treeldr::layout::Description::Option(
					value.into_required().unwrap().try_map_borrow_metadata(
						|item_layout_id, meta| {
							let item_layout_ref =
								context.require_layout_id(item_layout_id).map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::Option(None),
										meta.first().unwrap().value.into_metadata().clone(),
									)
								})?;
							Ok(treeldr::layout::Optional::new(Meta(
								item_layout_ref,
								meta.first().unwrap().value.into_metadata().clone(),
							)))
						},
					)?,
				))
			}
			Self::Set(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Set(None))
				})?;
				Ok(treeldr::layout::Description::Set(
					value.into_required().unwrap().try_map_borrow_metadata(
						|item_layout_id, meta| {
							let meta = meta.first().unwrap().value.into_metadata();
							let item_layout_ref =
								context.require_layout_id(item_layout_id).map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::Set(None),
										meta.clone(),
									)
								})?;
							Ok(treeldr::layout::Set::new(
								Meta(item_layout_ref, meta.clone()),
								restrictions.map(Restrictions::into_container),
							))
						},
					)?,
				))
			}
			Self::Map(p) => {
				let key_layout = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Map(None))
				})?;

				let value_layout = map_value.into_required_layout_at_node_binding(
					context,
					id,
					Property::MapValue(None),
					self.metadata(),
				)?;

				Ok(treeldr::layout::Description::Map(
					key_layout
						.into_required()
						.unwrap()
						.try_map_borrow_metadata(|key_layout_id, meta| {
							let meta = meta.first().unwrap().value.into_metadata();
							let item_layout_ref =
								context.require_layout_id(key_layout_id).map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::Map(None),
										meta.clone(),
									)
								})?;
							Ok(treeldr::layout::Map::new(
								Meta(item_layout_ref, meta.clone()),
								value_layout,
							))
						})?,
				))
			}
			Self::OneOrMany(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::OneOrMany(None))
				})?;
				Ok(treeldr::layout::Description::OneOrMany(
					value.into_required().unwrap().try_map_borrow_metadata(
						|item_layout_id, meta| {
							let meta = meta.first().unwrap().value.into_metadata();
							let item_layout_ref =
								context.require_layout_id(item_layout_id).map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::OneOrMany(None),
										meta.clone(),
									)
								})?;
							Ok(treeldr::layout::OneOrMany::new(
								Meta(item_layout_ref, meta.clone()),
								restrictions.map(Restrictions::into_container),
							))
						},
					)?,
				))
			}
			Self::Array(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Array(None))
				})?;
				Ok(treeldr::layout::Description::Array(
					value.into_required().unwrap().try_map_borrow_metadata(
						|item_layout_id, meta| {
							let meta = meta.first().unwrap().value.into_metadata();
							let item_layout_ref =
								context.require_layout_id(item_layout_id).map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::Array(None),
										meta.clone(),
									)
								})?;
							let semantics = array_semantics.clone().build(context, id)?;
							Ok(treeldr::layout::Array::new(
								Meta(item_layout_ref, meta.clone()),
								restrictions.map(Restrictions::into_container),
								semantics,
							))
						},
					)?,
				))
			}
			Self::Alias(p) => {
				let value = p.clone().try_unwrap().map_err(|e| {
					e.at_functional_node_property(id, DescriptionProperty::Alias(None))
				})?;
				Ok(treeldr::layout::Description::Alias(
					value.into_required().unwrap().try_map_borrow_metadata(
						|alias_layout_id, meta| {
							let meta = meta.first().unwrap().value.into_metadata();
							let alias_layout_ref =
								context.require_layout_id(alias_layout_id).map_err(|e| {
									e.at_node_property(
										id,
										DescriptionProperty::Alias(None),
										meta.clone(),
									)
								})?;
							Ok(alias_layout_ref)
						},
					)?,
				))
			}
		}
	}
}

pub struct NoSingleDescription<'a, M> {
	a: SingleDescriptionPropertyRef<'a, M>,
	b: SingleDescriptionPropertyRef<'a, M>,
}

/// Layout description properties.
#[derive(Debug, Clone, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DescriptionProperties<M> {
	pub derived_from: FunctionalPropertyValue<Id, M>,
	pub struct_: FunctionalPropertyValue<Id, M>,
	pub reference: FunctionalPropertyValue<Id, M>,
	pub enum_: FunctionalPropertyValue<Id, M>,
	pub required: FunctionalPropertyValue<Id, M>,
	pub option: FunctionalPropertyValue<Id, M>,
	pub set: FunctionalPropertyValue<Id, M>,
	pub map: FunctionalPropertyValue<Id, M>,
	pub one_or_many: FunctionalPropertyValue<Id, M>,
	pub array: FunctionalPropertyValue<Id, M>,
	pub alias: FunctionalPropertyValue<Id, M>,
}

impl<M> DescriptionProperties<M> {
	pub fn is_empty(&self) -> bool {
		self.derived_from.is_empty()
			&& self.struct_.is_empty()
			&& self.reference.is_empty()
			&& self.enum_.is_empty()
			&& self.required.is_empty()
			&& self.option.is_empty()
			&& self.set.is_empty()
			&& self.map.is_empty()
			&& self.one_or_many.is_empty()
			&& self.array.is_empty()
			&& self.alias.is_empty()
	}

	pub fn insert_base(&mut self, Meta(b, m): Meta<BaseDescriptionBinding, M>)
	where
		M: Merge,
	{
		match b {
			BaseDescriptionBinding::DerivedFrom(id) => self.derived_from.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Struct(id) => self.struct_.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Reference(id) => self.reference.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Enum(id) => self.enum_.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Required(id) => self.required.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Option(id) => self.option.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Set(id) => self.set.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Map(id) => self.map.insert_base(Meta(id, m)),
			BaseDescriptionBinding::OneOrMany(id) => self.one_or_many.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Array(id) => self.array.insert_base(Meta(id, m)),
			BaseDescriptionBinding::Alias(id) => self.alias.insert_base(Meta(id, m)),
		}
	}

	pub fn set(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		prop: DescriptionProperty,
		value: Meta<Value, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			DescriptionProperty::Alias(p) => {
				self.alias.insert(p, prop_cmp, value.into_expected_id()?)
			}
			DescriptionProperty::Array(p) => {
				self.array.insert(p, prop_cmp, value.into_expected_id()?)
			}
			DescriptionProperty::DerivedFrom(p) => {
				self.derived_from
					.insert(p, prop_cmp, value.into_expected_id()?)
			}
			DescriptionProperty::Fields(p) => {
				self.struct_.insert(p, prop_cmp, value.into_expected_id()?)
			}
			DescriptionProperty::OneOrMany(p) => {
				self.one_or_many
					.insert(p, prop_cmp, value.into_expected_id()?)
			}
			DescriptionProperty::Option(p) => {
				self.option.insert(p, prop_cmp, value.into_expected_id()?)
			}
			DescriptionProperty::Reference(p) => {
				self.reference
					.insert(p, prop_cmp, value.into_expected_id()?)
			}
			DescriptionProperty::Required(p) => {
				self.required.insert(p, prop_cmp, value.into_expected_id()?)
			}
			DescriptionProperty::Set(p) => self.set.insert(p, prop_cmp, value.into_expected_id()?),
			DescriptionProperty::Map(p) => self.map.insert(p, prop_cmp, value.into_expected_id()?),
			DescriptionProperty::Variants(p) => {
				self.enum_.insert(p, prop_cmp, value.into_expected_id()?)
			}
		}

		Ok(())
	}

	pub fn is_equivalent_to(&self, context: &Context<M>, other: &Self) -> bool {
		self.is_included_in(context, other) && other.is_included_in(context, self)
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
			bs: [&FunctionalPropertyValue<Id, M>; N],
		) -> bool {
			if bs.iter().all(|b| b.is_empty()) && !a.is_empty() {
				false
			} else {
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
			}
		}

		check(context, is_eq, &self.alias, [&other.alias])
			&& check(context, is_included_in, &self.reference, [&other.reference])
			&& check(
				context,
				is_included_in,
				&self.required,
				[&other.required, &other.option, &other.one_or_many],
			) && check(
			context,
			is_included_in,
			&self.option,
			[&other.option, &other.one_or_many],
		) && check(
			context,
			is_included_in,
			&self.one_or_many,
			[&other.one_or_many],
		) && check(context, is_included_in, &self.array, [&other.array])
			&& check(context, is_included_in, &self.set, [&other.set])
			&& check(context, is_struct_included, &self.struct_, [&other.struct_])
			&& check(context, is_enum_included, &self.enum_, [&other.enum_])
	}

	pub fn collect_sub_layouts(&self, context: &Context<M>, sub_layouts: &mut Vec<SubLayout<M>>)
	where
		M: Clone,
	{
		for PropertyValueRef {
			value: Meta(fields_id, _),
			..
		} in &self.struct_
		{
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

	pub fn single_description(
		&self,
	) -> Result<Option<SingleDescriptionPropertyRef<M>>, NoSingleDescription<M>> {
		let mut result = None;

		if !self.derived_from.is_empty() {
			let a = SingleDescriptionPropertyRef::DerivedFrom(&self.derived_from);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.struct_.is_empty() {
			let a = SingleDescriptionPropertyRef::Struct(&self.struct_);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.reference.is_empty() {
			let a = SingleDescriptionPropertyRef::Reference(&self.reference);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.enum_.is_empty() {
			let a = SingleDescriptionPropertyRef::Enum(&self.enum_);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.required.is_empty() {
			let a = SingleDescriptionPropertyRef::Required(&self.required);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.option.is_empty() {
			let a = SingleDescriptionPropertyRef::Option(&self.option);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.one_or_many.is_empty() {
			let a = SingleDescriptionPropertyRef::OneOrMany(&self.one_or_many);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.array.is_empty() {
			let a = SingleDescriptionPropertyRef::Array(&self.array);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.set.is_empty() {
			let a = SingleDescriptionPropertyRef::Set(&self.set);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		if !self.alias.is_empty() {
			let a = SingleDescriptionPropertyRef::Alias(&self.alias);
			match result {
				Some(b) => return Err(NoSingleDescription { a, b }),
				None => result = Some(a),
			}
		}

		Ok(result)
	}

	pub fn build(
		&self,
		context: &Context<M>,
		id: Id,
		restrictions: MetaOption<Restrictions<M>, M>,
		array_semantics: &array::Semantics<M>,
		map_value: FunctionalPropertyValue<Id, M>,
	) -> Result<treeldr::layout::Description<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		match self.single_description() {
			Ok(Some(desc)) => desc.build(context, id, restrictions, array_semantics, map_value),
			Ok(None) => match Primitive::from_id(id) {
				Some(p) => Ok(treeldr::layout::Description::Primitive(p)),
				None => Ok(treeldr::layout::Description::Never),
			},
			Err(NoSingleDescription { a, b }) => Err(Meta(
				error::LayoutDescriptionMismatch {
					id,
					desc1: a.cloned(),
					desc2: b.cloned(),
				}
				.into(),
				a.metadata().clone(),
			)),
		}
	}

	pub fn iter(&self) -> DescriptionPropertiesIter<M> {
		DescriptionPropertiesIter {
			derived_from: self.derived_from.iter(),
			struct_: self.struct_.iter(),
			reference: self.reference.iter(),
			enum_: self.enum_.iter(),
			required: self.required.iter(),
			option: self.option.iter(),
			set: self.set.iter(),
			one_or_many: self.one_or_many.iter(),
			array: self.array.iter(),
			alias: self.alias.iter(),
		}
	}
}

impl<M: Merge> Extend<Meta<BaseDescriptionBinding, M>> for DescriptionProperties<M> {
	fn extend<T: IntoIterator<Item = Meta<BaseDescriptionBinding, M>>>(&mut self, iter: T) {
		for b in iter {
			self.insert_base(b)
		}
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
	alias: functional_property_value::Iter<'a, Id, M>,
}

impl<'a, M> Iterator for DescriptionPropertiesIter<'a, M> {
	type Item = Meta<DescriptionBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.derived_from
			.next()
			.map(|p| p.into_cloned_class_binding(DescriptionBinding::DerivedFrom))
			.or_else(|| {
				self.struct_
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::Struct))
			})
			.or_else(|| {
				self.reference
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::Reference))
			})
			.or_else(|| {
				self.enum_
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::Enum))
			})
			.or_else(|| {
				self.required
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::Required))
			})
			.or_else(|| {
				self.option
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::Option))
			})
			.or_else(|| {
				self.set
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::Set))
			})
			.or_else(|| {
				self.one_or_many
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::OneOrMany))
			})
			.or_else(|| {
				self.array
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::Array))
			})
			.or_else(|| {
				self.alias
					.next()
					.map(|p| p.into_cloned_class_binding(DescriptionBinding::Alias))
			})
	}
}

impl<M> MapIds for DescriptionProperties<M> {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		self.struct_
			.map_ids_in(Some(DescriptionProperty::Fields(None).into()), &f);
		self.reference
			.map_ids_in(Some(DescriptionProperty::Reference(None).into()), &f);
		self.enum_
			.map_ids_in(Some(DescriptionProperty::Variants(None).into()), &f);
		self.required
			.map_ids_in(Some(DescriptionProperty::Required(None).into()), &f);
		self.option
			.map_ids_in(Some(DescriptionProperty::Option(None).into()), &f);
		self.set
			.map_ids_in(Some(DescriptionProperty::Set(None).into()), &f);
		self.one_or_many
			.map_ids_in(Some(DescriptionProperty::OneOrMany(None).into()), &f);
		self.array
			.map_ids_in(Some(DescriptionProperty::Array(None).into()), &f);
		self.alias
			.map_ids_in(Some(DescriptionProperty::Alias(None).into()), f);
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BaseDescriptionBinding {
	DerivedFrom(Id),
	Struct(Id),
	Reference(Id),
	Enum(Id),
	Required(Id),
	Option(Id),
	Set(Id),
	Map(Id),
	OneOrMany(Id),
	Array(Id),
	Alias(Id),
}

#[derive(Debug, Clone)]
pub enum DescriptionBinding {
	DerivedFrom(Option<TId<UnknownProperty>>, Id),
	Struct(Option<TId<UnknownProperty>>, Id),
	Reference(Option<TId<UnknownProperty>>, Id),
	Enum(Option<TId<UnknownProperty>>, Id),
	Required(Option<TId<UnknownProperty>>, Id),
	Option(Option<TId<UnknownProperty>>, Id),
	Set(Option<TId<UnknownProperty>>, Id),
	Map(Option<TId<UnknownProperty>>, Id),
	OneOrMany(Option<TId<UnknownProperty>>, Id),
	Array(Option<TId<UnknownProperty>>, Id),
	Alias(Option<TId<UnknownProperty>>, Id),
}

impl DescriptionBinding {
	pub fn property(&self) -> DescriptionProperty {
		match self {
			Self::DerivedFrom(p, _) => DescriptionProperty::DerivedFrom(*p),
			Self::Reference(p, _) => DescriptionProperty::Reference(*p),
			Self::Struct(p, _) => DescriptionProperty::Fields(*p),
			Self::Enum(p, _) => DescriptionProperty::Variants(*p),
			Self::Required(p, _) => DescriptionProperty::Required(*p),
			Self::Option(p, _) => DescriptionProperty::Option(*p),
			Self::Set(p, _) => DescriptionProperty::Set(*p),
			Self::Map(p, _) => DescriptionProperty::Map(*p),
			Self::OneOrMany(p, _) => DescriptionProperty::OneOrMany(*p),
			Self::Array(p, _) => DescriptionProperty::Array(*p),
			Self::Alias(p, _) => DescriptionProperty::Alias(*p),
		}
	}

	pub fn value<'a>(&self) -> BindingValueRef<'a> {
		match self {
			Self::DerivedFrom(_, v) => BindingValueRef::Id(*v),
			Self::Reference(_, v) => BindingValueRef::Id(*v),
			Self::Struct(_, v) => BindingValueRef::Id(*v),
			Self::Enum(_, v) => BindingValueRef::Id(*v),
			Self::Required(_, v) => BindingValueRef::Id(*v),
			Self::Option(_, v) => BindingValueRef::Id(*v),
			Self::Set(_, v) => BindingValueRef::Id(*v),
			Self::Map(_, v) => BindingValueRef::Id(*v),
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
	/// Types for which this layout is defined.
	ty: PropertyValues<Id, M>,

	/// Layout description.
	desc: DescriptionProperties<M>,

	intersection_of: FunctionalPropertyValue<Id, M>,

	/// List of restrictions.
	restrictions: FunctionalPropertyValue<Id, M>,

	/// List semantics.
	array_semantics: array::Semantics<M>,

	/// Map layout value format.
	map_value: FunctionalPropertyValue<Id, M>,
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
			ty: PropertyValues::default(),
			desc: DescriptionProperties::default(),
			intersection_of: FunctionalPropertyValue::default(),
			restrictions: FunctionalPropertyValue::default(),
			array_semantics: array::Semantics::default(),
			map_value: FunctionalPropertyValue::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	/// Types for which the layout is defined.
	pub fn ty(&self) -> &PropertyValues<Id, M> {
		&self.ty
	}

	pub fn ty_mut(&mut self) -> &mut PropertyValues<Id, M> {
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
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		prop: Property,
		value: Meta<Value, M>,
	) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::For(p) => self.ty_mut().insert(p, prop_cmp, value.into_expected_id()?),
			Property::Description(prop) => self.desc.set(prop_cmp, prop, value)?,
			Property::WithRestrictions(p) => {
				self.restrictions_mut()
					.insert(p, prop_cmp, value.into_expected_id()?)
			}
			Property::IntersectionOf(p) => {
				self.intersection_of_mut()
					.insert(p, prop_cmp, value.into_expected_id()?)
			}
			Property::ArrayListFirst(p) => {
				self.array_semantics
					.set_first(p, prop_cmp, value.into_expected_id()?)
			}
			Property::ArrayListRest(p) => {
				self.array_semantics
					.set_rest(p, prop_cmp, value.into_expected_id()?)
			}
			Property::ArrayListNil(p) => {
				self.array_semantics
					.set_nil(p, prop_cmp, value.into_expected_id()?)
			}
			Property::MapValue(p) => self
				.map_value
				.insert(p, prop_cmp, value.into_expected_id()?),
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
			map_value: self.map_value.iter(),
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
		self.array_semantics.set_first_base(first_prop)
	}

	pub fn set_array_list_rest(&mut self, value: Meta<Id, M>) {
		self.array_semantics.set_rest_base(value)
	}

	pub fn set_array_list_nil(&mut self, value: Meta<Id, M>) {
		self.array_semantics.set_nil_base(value)
	}

	pub fn set_array_semantics(
		&mut self,
		prop_cmp: impl Fn(TId<UnknownProperty>, TId<UnknownProperty>) -> Option<Ordering>,
		value: array::Semantics<M>,
	) {
		self.array_semantics.unify_with(prop_cmp, value)
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
							name.push_name(field_name.value.value());

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

		for PropertyValueRef {
			value: Meta(list_id, meta),
			..
		} in &self.intersection_of
		{
			let list = context.require_list(*list_id).map_err(|e| {
				e.at_node_property(as_resource.id, Property::IntersectionOf(None), meta.clone())
			})?;

			let intersections = list.try_fold(
				context,
				Ok(None),
				|intersection: Result<Option<intersection::Definition<M>>, Incomplete>, items| {
					match intersection {
						Ok(intersection) => {
							let mut result = Vec::new();

							for PropertyValueRef { value, .. } in items {
								let Meta(layout_id, layout_metadata) =
									value.cloned().into_expected_id()?;

								let new_intersection = match intersection::Definition::from_id(
									context,
									Meta(layout_id, layout_metadata),
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
		_metadata: &M,
	) -> Result<treeldr::layout::Description<M>, Error<M>>
	where
		M: Merge,
	{
		let restrictions_id = self.restrictions.clone().try_unwrap().map_err(|e| {
			e.at_functional_node_property(as_resource.id, Property::WithRestrictions(None))
		})?;

		let restrictions = restrictions_id
			.as_required()
			.map(|restrictions_id| {
				let list_id = restrictions_id.value();
				let mut restrictions = Restrictions::default();
				let list = context.require_list(*list_id).map_err(|e| {
					e.at_node_property(
						as_resource.id,
						Property::WithRestrictions(None),
						restrictions_id.sub_property_metadata().clone(),
					)
				})?;
				for restriction_value in list.iter(context) {
					let Meta(restriction_id, meta) =
						restriction_value?.cloned().into_expected_id()?;
					let restriction_definition = context
						.require(restriction_id)
						.map_err(|e| e.at(meta.clone()))?
						.require_layout_restriction(context)
						.map_err(|e| e.at(meta.clone()))?;
					let restriction = restriction_definition.build()?;
					restrictions.insert(restriction)?
				}

				Ok(Meta(
					restrictions,
					restrictions_id.sub_property_metadata().clone(),
				))
			})
			.transpose()?
			.into();

		self.desc.build(
			context,
			as_resource.id,
			restrictions,
			&self.array_semantics,
			self.map_value.clone(),
		)
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
			.map_err(|e| {
				e.at_functional_node_property(as_resource.id, Property::IntersectionOf(None))
			})?
			.try_map_borrow_metadata(|id, meta| {
				let list = context.require_list(id).map_err(|e| {
					e.at_node_property(
						as_resource.id,
						Property::IntersectionOf(None),
						meta.first().unwrap().value.into_metadata().clone(),
					)
				})?;
				let mut intersection = Multiple::default();
				for item in list.iter(context) {
					let Meta(layout_id, layout_meta) = item?.cloned().into_expected_id()?;
					let layout_tid = context
						.require_layout_id(layout_id)
						.map_err(|e| e.at(layout_meta.clone()))?;
					intersection.insert(Meta(layout_tid, layout_meta))
				}

				Ok(intersection)
			})?;

		let desc = self.build_description(context, as_resource, as_component, &metadata)?;
		let ty = self
			.ty()
			.try_mapped(|_, Meta(ty, m)| context.require_type_id(*ty).map(|ty| Meta(ty, m.clone())))
			.map_err(|(Meta(e, m), _)| {
				e.at_node_property(as_resource.id, Property::For(None), m.clone())
			})?;

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
		self.ty.map_ids_in(Some(Property::For(None).into()), &f);
		self.desc.map_ids(&f);
		self.intersection_of
			.map_ids_in(Some(Property::IntersectionOf(None).into()), &f);
		self.restrictions
			.map_ids_in(Some(Property::WithRestrictions(None).into()), &f);
		self.array_semantics.map_ids(f)
	}
}

#[derive(Debug)]
pub enum ClassBinding {
	For(Option<TId<UnknownProperty>>, Id),
	Description(DescriptionBinding),
	IntersectionOf(Option<TId<UnknownProperty>>, Id),
	WithRestrictions(Option<TId<UnknownProperty>>, Id),
	ArraySemantics(array::Binding),
	MapValue(Option<TId<UnknownProperty>>, Id),
}

pub type Binding = ClassBinding;

impl ClassBinding {
	pub fn property(&self) -> Property {
		match self {
			Self::For(p, _) => Property::For(*p),
			Self::Description(d) => Property::Description(d.property()),
			Self::IntersectionOf(p, _) => Property::IntersectionOf(*p),
			Self::WithRestrictions(p, _) => Property::WithRestrictions(*p),
			Self::ArraySemantics(b) => b.property(),
			Self::MapValue(p, _) => Property::MapValue(*p),
		}
	}

	pub fn value<'a>(&self) -> BindingValueRef<'a> {
		match self {
			Self::For(_, v) => BindingValueRef::Id(*v),
			Self::Description(d) => d.value(),
			Self::IntersectionOf(_, v) => BindingValueRef::Id(*v),
			Self::WithRestrictions(_, v) => BindingValueRef::Id(*v),
			Self::ArraySemantics(b) => b.value(),
			Self::MapValue(_, v) => BindingValueRef::Id(*v),
		}
	}
}

pub struct ClassBindings<'a, M> {
	ty: functional_property_value::Iter<'a, Id, M>,
	desc: DescriptionPropertiesIter<'a, M>,
	intersection_of: functional_property_value::Iter<'a, Id, M>,
	restrictions: functional_property_value::Iter<'a, Id, M>,
	array_semantics: array::Bindings<'a, M>,
	map_value: functional_property_value::Iter<'a, Id, M>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBinding, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.ty
			.next()
			.map(|v| v.into_cloned_class_binding(ClassBinding::For))
			.or_else(|| {
				self.desc
					.next()
					.map(|m| m.map(ClassBinding::Description))
					.or_else(|| {
						self.intersection_of
							.next()
							.map(|v| v.into_cloned_class_binding(ClassBinding::IntersectionOf))
							.or_else(|| {
								self.restrictions
									.next()
									.map(|v| {
										v.into_cloned_class_binding(ClassBinding::WithRestrictions)
									})
									.or_else(|| {
										self.array_semantics
											.next()
											.map(|v| v.map(ClassBinding::ArraySemantics))
											.or_else(|| {
												self.map_value.next().map(|v| {
													v.into_cloned_class_binding(
														ClassBinding::MapValue,
													)
												})
											})
									})
							})
					})
			})
	}
}
