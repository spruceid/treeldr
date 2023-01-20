use crate::{
	context::{MapIds, MapIdsIn},
	error, rdf,
	resource::{self, BindingValueRef},
	single::{self, Conflict},
	utils::TryCollect,
	Context, Error, ObjectAsId, ObjectAsRequiredId, Single,
};
use locspan::Meta;
use rdf_types::IriVocabulary;
pub use treeldr::layout::{DescriptionProperty, Property};
use treeldr::{metadata::Merge, vocab::Object, Id, IriIndex, Multiple, Name};

pub mod array;
pub mod field;
pub mod intersection;
pub mod primitive;
pub mod restriction;
pub mod variant;

pub use primitive::Primitive;
pub use restriction::{Restriction, Restrictions};

use primitive::BuildPrimitive;

/// Layout description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Description {
	Never,
	Primitive(Primitive),
	Struct(Id),
	Reference(Id),
	Enum(Id),
	Required(Id),
	Option(Id),
	Set(Id),
	OneOrMany(Id),
	Array(Id),
	Alias(Id),
}

impl Description {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Never => Kind::Never,
			Self::Reference(_) => Kind::Reference,
			Self::Struct(_) => Kind::Struct,
			Self::Primitive(n) => Kind::Primitive(*n),
			Self::Enum(_) => Kind::Enum,
			Self::Required(_) => Kind::Required,
			Self::Option(_) => Kind::Option,
			Self::Set(_) => Kind::Set,
			Self::OneOrMany(_) => Kind::OneOrMany,
			Self::Array(_) => Kind::Array,
			Self::Alias(_) => Kind::Alias,
		}
	}

	pub fn into_binding(self) -> Option<DescriptionBinding> {
		match self {
			Self::Never | Self::Primitive(_) => None,
			Self::Struct(id) => Some(DescriptionBinding::Struct(id)),
			Self::Reference(id) => Some(DescriptionBinding::Reference(id)),
			Self::Enum(id) => Some(DescriptionBinding::Enum(id)),
			Self::Required(id) => Some(DescriptionBinding::Required(id)),
			Self::Option(id) => Some(DescriptionBinding::Option(id)),
			Self::Set(id) => Some(DescriptionBinding::Set(id)),
			Self::OneOrMany(id) => Some(DescriptionBinding::OneOrMany(id)),
			Self::Array(id) => Some(DescriptionBinding::Array(id)),
			Self::Alias(id) => Some(DescriptionBinding::Alias(id)),
		}
	}

	pub fn is_included_in<M>(&self, context: &Context<M>, other: &Self) -> bool {
		match (self, other) {
			(Self::Primitive(a), Self::Primitive(b)) => a == b,
			(Self::Alias(a), Self::Alias(b)) => a == b,
			(Self::Reference(a), Self::Reference(b)) => is_included_in(context, *a, *b),
			(Self::Struct(a), Self::Struct(b)) => {
				// all fields in `b` must include a field of `a`.
				match (context.get_list(*a), context.get_list(*b)) {
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
			(Self::Enum(a), Self::Enum(b)) => {
				// all variants in `a` must be included in a variant in `b`.
				match (context.get_list(*a), context.get_list(*b)) {
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
			(Self::Required(a), Self::Required(b) | Self::Option(b) | Self::OneOrMany(b)) => {
				is_included_in(context, *a, *b)
			}
			(Self::Option(a), Self::Option(b) | Self::OneOrMany(b)) => {
				is_included_in(context, *a, *b)
			}
			(Self::OneOrMany(a), Self::OneOrMany(b)) => is_included_in(context, *a, *b),
			(Self::Array(a), Self::Array(b)) => is_included_in(context, *a, *b),
			(Self::Set(a), Self::Set(b)) => is_included_in(context, *a, *b),
			_ => false,
		}
	}

	pub fn collect_sub_layouts<M>(
		&self,
		context: &Context<M>,
		sub_layouts: &mut Vec<SubLayout<M>>,
		metadata: &M,
	) where
		M: Clone,
	{
		match self {
			Self::Struct(fields_id) => {
				if let Some(fields) = context.get_list(*fields_id) {
					for Meta(object, _) in fields.lenient_iter(context) {
						if let Some(field_id) = object.as_id() {
							if let Some(field) = context
								.get(field_id)
								.map(resource::Definition::as_formatted)
							{
								for field_layout_id in field.format() {
									if let Some(field_layout) = context
										.get(**field_layout_id)
										.map(resource::Definition::as_layout)
									{
										sub_layouts.push(SubLayout {
											layout: field_layout_id.cloned(),
											connection: LayoutConnection::FieldContainer(field_id),
										});

										for Meta(container_desc, meta) in field_layout.description()
										{
											let item_layout_id = match container_desc {
												Description::Required(id) => *id,
												Description::Option(id) => *id,
												Description::Set(id) => *id,
												Description::OneOrMany(id) => *id,
												Description::Array(id) => *id,
												_ => panic!(
													"invalid field layout: not a container: {:?}",
													container_desc
												),
											};

											sub_layouts.push(SubLayout {
												layout: Meta(item_layout_id, meta.clone()),
												connection: LayoutConnection::FieldItem(field_id),
											});
										}
									}
								}
							}
						}
					}
				}
			}
			Self::Set(item_layout_id) => sub_layouts.push(SubLayout {
				layout: Meta(*item_layout_id, metadata.clone()),
				connection: LayoutConnection::Item,
			}),
			Self::OneOrMany(item_layout_id) => sub_layouts.push(SubLayout {
				layout: Meta(*item_layout_id, metadata.clone()),
				connection: LayoutConnection::Item,
			}),
			Self::Array(item_layout_id) => sub_layouts.push(SubLayout {
				layout: Meta(*item_layout_id, metadata.clone()),
				connection: LayoutConnection::Item,
			}),
			_ => (),
		}
	}
}

impl MapIds for Description {
	fn map_ids(&mut self, f: impl Fn(Id, Option<crate::Property>) -> Id) {
		match self {
			Self::Never | Self::Primitive(_) => (),
			Self::Struct(id) => id.map_ids_in(Some(DescriptionProperty::Fields.into()), f),
			Self::Reference(id) => id.map_ids_in(Some(DescriptionProperty::Reference.into()), f),
			Self::Enum(id) => id.map_ids_in(Some(DescriptionProperty::Variants.into()), f),
			Self::Required(id) => id.map_ids_in(Some(DescriptionProperty::Required.into()), f),
			Self::Option(id) => id.map_ids_in(Some(DescriptionProperty::Option.into()), f),
			Self::Set(id) => id.map_ids_in(Some(DescriptionProperty::Set.into()), f),
			Self::OneOrMany(id) => id.map_ids_in(Some(DescriptionProperty::OneOrMany.into()), f),
			Self::Array(id) => id.map_ids_in(Some(DescriptionProperty::Array.into()), f),
			Self::Alias(id) => id.map_ids_in(Some(DescriptionProperty::Alias.into()), f),
		}
	}
}

pub enum DescriptionBinding {
	Struct(Id),
	Reference(Id),
	Enum(Id),
	Required(Id),
	Option(Id),
	Set(Id),
	OneOrMany(Id),
	Array(Id),
	Alias(Id),
}

impl DescriptionBinding {
	pub fn property(&self) -> DescriptionProperty {
		match self {
			Self::Reference(_) => DescriptionProperty::Reference,
			Self::Struct(_) => DescriptionProperty::Fields,
			Self::Enum(_) => DescriptionProperty::Variants,
			Self::Required(_) => DescriptionProperty::Required,
			Self::Option(_) => DescriptionProperty::Option,
			Self::Set(_) => DescriptionProperty::Set,
			Self::OneOrMany(_) => DescriptionProperty::OneOrMany,
			Self::Array(_) => DescriptionProperty::Array,
			Self::Alias(_) => DescriptionProperty::Alias,
		}
	}

	pub fn value<'a, M>(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::Reference(v) => BindingValueRef::Id(*v),
			Self::Struct(v) => BindingValueRef::Id(*v),
			Self::Enum(v) => BindingValueRef::Id(*v),
			Self::Required(v) => BindingValueRef::Id(*v),
			Self::Option(v) => BindingValueRef::Id(*v),
			Self::Set(v) => BindingValueRef::Id(*v),
			Self::OneOrMany(v) => BindingValueRef::Id(*v),
			Self::Array(v) => BindingValueRef::Id(*v),
			Self::Alias(v) => BindingValueRef::Id(*v),
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
	ty: Single<Id, M>,

	/// Layout description.
	desc: Single<Description, M>,

	intersection_of: Single<Id, M>,

	/// List of restrictions.
	restrictions: Single<Id, M>,

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
			ty: Single::default(),
			desc: Single::default(),
			intersection_of: Single::default(),
			restrictions: Single::default(),
			array_semantics: array::Semantics::default(),
		}
	}
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> &Single<Id, M> {
		&self.ty
	}

	pub fn ty_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.ty
	}

	pub fn description(&self) -> &Single<Description, M> {
		&self.desc
	}

	pub fn description_mut(&mut self) -> &mut Single<Description, M> {
		&mut self.desc
	}

	pub fn intersection_of(&self) -> &Single<Id, M> {
		&self.intersection_of
	}

	pub fn intersection_of_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.intersection_of
	}

	pub fn restrictions(&self) -> &Single<Id, M> {
		&self.restrictions
	}

	pub fn restrictions_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.restrictions
	}

	pub fn is_included_in(&self, context: &Context<M>, other: &Self) -> bool {
		self.desc.iter().all(|Meta(a, _)| {
			other
				.desc
				.iter()
				.all(|Meta(b, _)| a.is_included_in(context, b))
		})
	}

	pub fn set(&mut self, prop: Property, value: Meta<Object<M>, M>) -> Result<(), Error<M>>
	where
		M: Merge,
	{
		match prop {
			Property::For => self.ty_mut().insert(rdf::from::expect_id(value)?),
			Property::Description(DescriptionProperty::Alias) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::Alias(id))),
			Property::Description(DescriptionProperty::Array) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::Array(id))),
			Property::Description(DescriptionProperty::DerivedFrom) => {
				let Meta(id, meta) = rdf::from::expect_id(value)?;
				match Primitive::from_id(id) {
					Some(p) => self
						.description_mut()
						.insert(Meta(Description::Primitive(p), meta)),
					None => panic!("invalid primitive layout"),
				}
			}
			Property::Description(DescriptionProperty::Fields) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::Struct(id))),
			Property::Description(DescriptionProperty::OneOrMany) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::OneOrMany(id))),
			Property::Description(DescriptionProperty::Option) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::Option(id))),
			Property::Description(DescriptionProperty::Reference) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::Reference(id))),
			Property::Description(DescriptionProperty::Required) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::Required(id))),
			Property::Description(DescriptionProperty::Set) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::Set(id))),
			Property::Description(DescriptionProperty::Variants) => self
				.description_mut()
				.insert(rdf::from::expect_id(value)?.map(|id| Description::Enum(id))),
			Property::WithRestrictions => {
				self.restrictions_mut().insert(rdf::from::expect_id(value)?)
			}
			Property::IntersectionOf => self
				.intersection_of_mut()
				.insert(rdf::from::expect_id(value)?),
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
	pub fn set_primitive(&mut self, primitive: Meta<Primitive, M>) {
		self.desc.insert(primitive.map(Description::Primitive))
	}

	pub fn set_alias(&mut self, alias: Meta<Id, M>) {
		self.desc.insert(alias.map(Description::Alias))
	}

	pub fn set_fields(&mut self, fields: Meta<Id, M>) {
		self.desc.insert(fields.map(Description::Struct))
	}

	pub fn set_reference(&mut self, ty: Meta<Id, M>) {
		self.desc.insert(ty.map(Description::Reference))
	}

	pub fn set_enum(&mut self, variants: Meta<Id, M>) {
		self.desc.insert(variants.map(Description::Enum))
	}

	pub fn set_required(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(Description::Required))
	}

	pub fn set_option(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(Description::Option))
	}

	pub fn set_set(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(Description::Set))
	}

	pub fn set_one_or_many(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(Description::OneOrMany))
	}

	pub fn set_array(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(Description::Array))
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

		for Meta(desc, metadata) in &self.desc {
			desc.collect_sub_layouts(context, &mut sub_layouts, metadata)
		}

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
		// 	if let Some(singleton) = regexp.as_singleton() {
		// 		if let Ok(singleton_name) = Name::new(singleton) {
		// 			let mut name = Name::new("const").unwrap();
		// 			name.push_name(&singleton_name);
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
							let mut name = parent_layout_name.into_value().clone();
							name.push_name(field_name.value());

							return Some(Meta(name, as_resource.metadata.clone()));
						}
					}
					LayoutConnection::Item => {
						let mut name = parent_layout_name.into_value().clone();
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

		for Meta(list_id, meta) in &self.intersection_of {
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

							for Meta(object, layout_metadata) in items {
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
			.as_ref()
			.map(|Meta(list_id, meta)| {
				let mut restrictions = Restrictions::default();
				let list = context.require_list(*list_id).map_err(|e| {
					e.at_node_property(as_resource.id, Property::WithRestrictions, meta.clone())
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
			self.desc
				.clone()
				.try_unwrap()
				.map_err(|Conflict(Meta(desc1, meta), desc2)| {
					Meta(
						error::LayoutDescriptionMismatch {
							id: as_resource.id,
							desc1,
							desc2,
						}
						.into(),
						meta,
					)
				})?;

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
	ty: single::Iter<'a, Id, M>,
	desc: single::Iter<'a, Description, M>,
	intersection_of: single::Iter<'a, Id, M>,
	restrictions: single::Iter<'a, Id, M>,
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
