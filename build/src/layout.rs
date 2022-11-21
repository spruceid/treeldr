use crate::{error, utils::TryCollect, Context, Error, Single, single::Conflict, Node, ObjectAsId, ObjectAsRequiredId, resource};
use locspan::Meta;
use rdf_types::IriVocabulary;
use treeldr::{metadata::Merge, Id, IriIndex, Name};
pub use treeldr::layout::{Property, DescriptionProperty};

pub mod array;
pub mod field;
pub mod primitive;
pub mod restriction;
pub mod variant;
pub mod intersection;

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
}

impl Description {
	pub fn collect_sub_layouts<M>(
		&self,
		context: &Context<M>,
		sub_layouts: &mut Vec<SubLayout<M>>,
		metadata: &M,
	) where M: Clone {
		match self {
			Self::Struct(fields_id) => {
				if let Some(fields) = context.get_list(*fields_id) {
					for Meta(object, _) in fields.lenient_iter(context) {
						if let Some(field_id) = object.as_id() {
							if let Some(field) = context.get(field_id).map(Node::as_formatted) {
								for field_layout_id in field.format() {
									if let Some(field_layout) = context.get(**field_layout_id).map(Node::as_layout) {
										sub_layouts.push(SubLayout {
											layout: field_layout_id.cloned(),
											connection: LayoutConnection::FieldContainer(field_id),
										});
					
										for Meta(container_desc, meta) in field_layout.description() {
											let item_layout_id = match container_desc {
												Description::Required(id) => *id,
												Description::Option(id) => *id,
												Description::Set(id) => *id,
												Description::OneOrMany(id) => *id,
												Description::Array(id) => *id,
												_ => panic!("invalid field layout: not a container"),
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
	array_semantics: array::Semantics<M>
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

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self {
			ty: Single::default(),
			desc: Single::default(),
			intersection_of: Single::default(),
			restrictions: Single::default(),
			array_semantics: array::Semantics::default()
		}
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
}

impl<M: Merge> Definition<M> {
	pub fn set_primitive(&mut self, primitive: Meta<Primitive, M>) {
		self.desc.insert(primitive.map(|d| Description::Primitive(d)))
	}

	pub fn set_alias(&mut self, alias: Meta<Id, M>) {
		self.desc.insert(alias.map(|d| Description::Alias(d)))
	}

	pub fn set_fields(&mut self, fields: Meta<Id, M>) {
		self.desc.insert(fields.map(|d| Description::Struct(d)))
	}

	pub fn set_reference(&mut self, ty: Meta<Id, M>) {
		self.desc.insert(ty.map(|d| Description::Reference(d)))
	}

	pub fn set_enum(&mut self, variants: Meta<Id, M>) {
		self.desc.insert(variants.map(|d| Description::Enum(d)))
	}

	pub fn set_required(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Required(d)))
	}

	pub fn set_option(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Option(d)))
	}

	pub fn set_set(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Set(d)))
	}

	pub fn set_one_or_many(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::OneOrMany(d)))
	}

	pub fn set_array(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Array(d)))
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
		as_resource: &resource::Data<M>
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
							name.push_name(*field_name);

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
		as_resource: &treeldr::node::Data<M>,
	) -> Result<Option<intersection::Definition<M>>, Error<M>> where M: Merge {
		let mut result = intersection::Definition::default();

		#[derive(Debug, Clone, Copy)]
		struct Incomplete;

		for Meta(list_id, meta) in &self.intersection_of {
			let list = context.require_list(*list_id).map_err(|e| e.at_node_property(as_resource.id, Property::IntersectionOf, meta.clone()))?;
		
			let intersections = list.try_fold(context, Ok(None), |intersection: Result<Option<intersection::Definition<M>>, Incomplete>, items| {
				match intersection {
					Ok(intersection) => {
						let mut result = Vec::new();

						for Meta(object, layout_metadata) in items {
							let layout_id = object.as_required_id(&layout_metadata)?;

							let new_intersection = match intersection::Definition::from_id(context, layout_id, layout_metadata)? {
								Some(desc) => {
									Some(match &intersection {
										Some(intersection) => {
											let mut new_intersection = intersection.clone();
											new_intersection.intersect_with(desc);
											new_intersection
										}
										None => desc
									})
								}
								None => {
									return Ok(vec![Err(Incomplete)])
								}
							};

							result.push(Ok(new_intersection))
						}

						Ok(result)
					}
					Err(Incomplete) => Ok(vec![Err(Incomplete)])
				}
			});

			for intersection in intersections {
				match intersection? {
					Err(Incomplete) => return Ok(None),
					Ok(Some(def)) => result.add(def),
					Ok(None) => result.add_never(meta.clone())
				}
			}
		}

		Ok(Some(result))
	}

	pub fn build_description(
		&self,
		context: &crate::Context<M>,
		as_resource: &treeldr::node::Data<M>,
		_as_component: &treeldr::component::Data<M>,
		metadata: &M
	) -> Result<Meta<treeldr::layout::Description<M>, M>, Error<M>> where M: Merge {
		let restrictions_id = self.restrictions.clone().try_unwrap().map_err(|e| e.at_functional_node_property(as_resource.id, Property::WithRestrictions))?;

		let restrictions = restrictions_id.as_ref().map(|Meta(list_id, meta)| {
			let mut restrictions = Restrictions::default();
			let list = context.require_list(*list_id).map_err(|e| e.at_node_property(as_resource.id, Property::WithRestrictions, meta.clone()))?;
			for restriction_value in list.iter(context) {
				let Meta(restriction_value, meta) = restriction_value?;
				let restriction_id = restriction_value.as_required_id(meta)?;
				let restriction_definition = context.require(restriction_id)
					.map_err(|e| e.at(meta.clone()))?
					.require_layout_restriction(context)
					.map_err(|e| e.at(meta.clone()))?;
				let restriction = restriction_definition.build()?;
				restrictions.insert(restriction)?
			}

			Ok(restrictions)
		}).transpose()?.unwrap_or_default();

		let Meta(desc, desc_metadata) = self.desc.clone()
			.try_unwrap()
			.map_err(|Conflict(Meta(desc1, meta), desc2)| {
				Meta(
					error::LayoutDescriptionMismatch {
						id: as_resource.id,
						desc1,
						desc2
					}.into(),
					meta
				)
			})?
			.ok_or_else(|| {
				Meta(
					error::LayoutDescriptionMissing(as_resource.id).into(),
					metadata.clone(),
				)
			})?;

		let desc = match desc {
			Description::Never => treeldr::layout::Description::Never,
			Description::Primitive(n) => treeldr::layout::Description::Primitive(
				n.build(as_resource.id, restrictions.into_primitive(), &desc_metadata)?
			),
			Description::Reference(layout_id) => {
				let layout_ref = context.require_layout_id(layout_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::Reference, desc_metadata.clone()))?;
				let r = treeldr::layout::Reference::new(Meta(layout_ref, desc_metadata.clone()));
				treeldr::layout::Description::Reference(r)
			}
			Description::Struct(fields_id) => {
				let fields = context
					.require_list(fields_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::Fields, desc_metadata.clone()))?
					.iter(context)
					.map(|item| {
						let Meta(object, field_metadata) = item?.cloned();
						let field_id = object.into_required_id(&field_metadata)?;
						Ok(Meta(context.require_layout_field_id(field_id).map_err(|e| e.at(field_metadata.clone()))?, field_metadata))
					})
					.try_collect()?;

				let strct = treeldr::layout::Struct::new(fields);
				treeldr::layout::Description::Struct(strct)
			}
			Description::Enum(options_id) => {
				let variants: Vec<_> = context
					.require_list(options_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::Variants, desc_metadata.clone()))?
					.iter(context)
					.map(|item| {
						let Meta(object, variant_metadata) = item?.cloned();
						let variant_id = object.into_required_id(&variant_metadata)?;
						Ok(Meta(context.require_layout_variant_id(variant_id).map_err(|e| e.at(variant_metadata.clone()))?, variant_metadata))
					})
					.try_collect()?;

				let enm = treeldr::layout::Enum::new(variants);
				treeldr::layout::Description::Enum(enm)
			}
			Description::Required(item_layout_id) => {
				let item_layout_ref = context.require_layout_id(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::Required, desc_metadata.clone()))?;
				treeldr::layout::Description::Required(
					treeldr::layout::Required::new(Meta(item_layout_ref, desc_metadata.clone())),
				)
			}
			Description::Option(item_layout_id) => {
				let item_layout_ref = context.require_layout_id(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::Option, desc_metadata.clone()))?;
				treeldr::layout::Description::Option(
					treeldr::layout::Optional::new(Meta(item_layout_ref, desc_metadata.clone())),
				)
			}
			Description::Set(item_layout_id) => {
				let item_layout_ref = context.require_layout_id(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::Set, desc_metadata.clone()))?;
				treeldr::layout::Description::Set(
					treeldr::layout::Set::new(Meta(item_layout_ref, desc_metadata.clone()), restrictions.into_container()),
				)
			}
			Description::OneOrMany(item_layout_id) => {
				let item_layout_ref = context.require_layout_id(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::OneOrMany, desc_metadata.clone()))?;
				treeldr::layout::Description::OneOrMany(
					treeldr::layout::OneOrMany::new(
						Meta(item_layout_ref, desc_metadata.clone()),
						restrictions.into_container(),
					),
				)
			}
			Description::Array(item_layout_id) => {
				let item_layout_ref = context.require_layout_id(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::Array, desc_metadata.clone()))?;
				let semantics = self.array_semantics.clone().build(context, as_resource.id)?;
				treeldr::layout::Description::Array(
					treeldr::layout::Array::new(
						Meta(item_layout_ref, desc_metadata.clone()),
						restrictions.into_container(),
						semantics
					)
				)
			},
			Description::Alias(alias_layout_id) => {
				let alias_layout_ref = context.require_layout_id(alias_layout_id).map_err(|e| e.at_node_property(as_resource.id, DescriptionProperty::Alias, desc_metadata.clone()))?;
				treeldr::layout::Description::Alias(alias_layout_ref)
			}
		};

		Ok(Meta(desc, desc_metadata))
	}

	pub(crate) fn build(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		as_component: &treeldr::component::Data<M>,
		metadata: M,
	) -> Result<Meta<treeldr::layout::Definition<M>, M>, Error<M>> where M: Merge {
		let desc = self.build_description(context, as_resource, as_component, &metadata)?;
		let ty = self.ty.clone().into_type_at_node_binding(context, as_resource.id, Property::For)?;

		Ok(Meta(
			treeldr::layout::Definition::new(
				ty, desc
			),
			metadata
		))
	}
}

pub enum BindingRef<'a> {
	For(Id),
	Name(&'a Name),
	Description(Description),
}

// pub struct Bindings<'a> {
// 	name: Option<&'a Name>,
// 	ty: Option<Id>,
// 	desc: Option<Description>,
// 	restrictions: bool,
// }

// impl<'a> Iterator for Bindings<'a> {
// 	type Item = BindingRef<'a>;

// 	fn next(&mut self) -> Option<Self::Item> {
// 		self.name
// 			.take()
// 			.map(BindingRef::Name)
// 			.or_else(|| self.format.take().map(BindingRef::Format))
// 	}
// }
