use crate::{error::{self, NodeBindingMissing}, utils::TryCollect, Context, Error, ObjectToId, Single, node, single::Conflict, Node, resource, component};
use locspan::Meta;
use rdf_types::IriVocabulary;
use treeldr::{metadata::Merge, Id, IriIndex, Name};

pub mod array;
pub mod field;
pub mod primitive;
pub mod restriction;
pub mod variant;

pub use primitive::Primitive;
pub use restriction::{Restriction, Restrictions};

use primitive::BuildPrimitive;

/// Simple layout description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SimpleDescription {
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

impl SimpleDescription {
	pub fn kind(&self) -> SimpleKind {
		match self {
			Self::Never => SimpleKind::Never,
			Self::Reference(_) => SimpleKind::Reference,
			Self::Struct(_) => SimpleKind::Struct,
			Self::Primitive(n) => SimpleKind::Primitive(*n),
			Self::Enum(_) => SimpleKind::Enum,
			Self::Required(_) => SimpleKind::Required,
			Self::Option(_) => SimpleKind::Option,
			Self::Set(_) => SimpleKind::Set,
			Self::OneOrMany(_) => SimpleKind::OneOrMany,
			Self::Array(_) => SimpleKind::Array,
			Self::Alias(_) => SimpleKind::Alias,
		}
	}
}

impl SimpleDescription {
	pub fn sub_layouts<M>(
		&self,
		context: &Context<M>,
		metadata: &M,
	) -> Vec<SubLayout<M>>
	where
		M: Clone,
	{
		let mut sub_layouts = Vec::new();

		match self {
			Self::Struct(fields_id) => {
				if let Some(fields) = context.get_list(*fields_id) {
					for Meta(object, metadata) in fields.lenient_iter(context) {
						if let Some(field_id) = object.as_id() {
							if let Some(field) = context.get(field_id).and_then(Node::as_layout_field) {
								for field_layout_id in field.layout() {
									if let Some(field_layout) = context.get(**field_layout_id).and_then(Node::as_layout) {
										sub_layouts.push(SubLayout {
											layout: field_layout_id.cloned(),
											connection: LayoutConnection::FieldContainer(field_id),
										});
					
										for Meta(container_desc, meta) in field_layout.description() {
											let item_layout_id = match container_desc {
												Description::Simple(SimpleDescription::Required(id)) => *id,
												Description::Simple(SimpleDescription::Option(id)) => *id,
												Description::Simple(SimpleDescription::Set(id)) => *id,
												Description::Simple(SimpleDescription::OneOrMany(id)) => *id,
												Description::Simple(SimpleDescription::Array(id)) => *id,
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

		sub_layouts
	}
}

/// Layout description.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Description {
	Simple(SimpleDescription),
	Intersection(Id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SimpleKind {
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

/// Layout kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
	Simple(SimpleKind),
	Intersection
}

impl Description {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Simple(s) => Kind::Simple(s.kind()),
			Self::Intersection(_) => Kind::Intersection
		}
	}

	pub fn dependencies<M>(
		&self,
	) -> Vec<Id> {
		Vec::new()
	}
}

/// Layout definition.
#[derive(Clone)]
pub struct Definition<M> {
	/// Type for which this layout is defined.
	ty: Single<Id, M>,

	/// Layout description.
	desc: Single<Description, M>,

	/// List of restrictions.
	restrictions: Single<Id, M>,

	/// List semantics.
	array_semantics: array::Semantics<M>,

	/// Simple description.
	simple_desc: Option<Meta<SimpleDescription, M>>
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
			restrictions: Single::default(),
			array_semantics: array::Semantics::default(),
			simple_desc: None
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

	pub fn restrictions(&self) -> &Single<Id, M> {
		&self.restrictions
	}

	pub fn restrictions_mut(&mut self) -> &mut Single<Id, M> {
		&mut self.restrictions
	}
}

impl<M: Merge> Definition<M> {
	pub fn set_primitive(&mut self, primitive: Meta<Primitive, M>) {
		self.desc.insert(primitive.map(|d| Description::Simple(SimpleDescription::Primitive(d))))
	}

	pub fn set_alias(&mut self, alias: Meta<Id, M>) {
		self.desc.insert(alias.map(|d| Description::Simple(SimpleDescription::Alias(d))))
	}

	pub fn set_fields(&mut self, fields: Meta<Id, M>) {
		self.desc.insert(fields.map(|d| Description::Simple(SimpleDescription::Struct(d))))
	}

	pub fn set_reference(&mut self, ty: Meta<Id, M>) {
		self.desc.insert(ty.map(|d| Description::Simple(SimpleDescription::Reference(d))))
	}

	pub fn set_enum(&mut self, variants: Meta<Id, M>) {
		self.desc.insert(variants.map(|d| Description::Simple(SimpleDescription::Enum(d))))
	}

	pub fn set_required(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Simple(SimpleDescription::Required(d))))
	}

	pub fn set_option(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Simple(SimpleDescription::Option(d))))
	}

	pub fn set_set(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Simple(SimpleDescription::Set(d))))
	}

	pub fn set_one_or_many(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Simple(SimpleDescription::OneOrMany(d))))
	}

	pub fn set_array(&mut self, item: Meta<Id, M>) {
		self.desc.insert(item.map(|d| Description::Simple(SimpleDescription::Array(d))))
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
		let desc = self.simple_desc.as_ref().unwrap();
		desc.sub_layouts(context, desc.metadata())
	}

	pub fn dependencies(
		&self
	) -> Result<Vec<Id>, Error<M>> {
		let mut dependencies = Vec::new();

		for desc in &self.desc {
			dependencies.extend(desc.dependencies())
		}

		Ok(dependencies)
	}

	/// Build a default name for this layout.
	pub fn default_name(
		&self,
		context: &Context<M>,
		vocabulary: &impl IriVocabulary<Iri = IriIndex>,
		id: Id,
		parent_layouts: &[Meta<ParentLayout, M>],
		metadata: M,
	) -> Option<Meta<Name, M>> {
		if let Id::Iri(term) = id {
			if let Ok(Some(name)) = Name::from_iri(vocabulary.iri(&term).unwrap()) {
				return Some(Meta(name, metadata));
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
			let parent_layout = context.get(parent.layout).unwrap().as_resource();

			if let Some(parent_layout_name) = parent_layout.name().first() {
				match parent.connection {
					LayoutConnection::FieldItem(field_id) => {
						let field = context.get(field_id).unwrap().as_layout_field().unwrap();

						if let Some(field_name) = field.name().first() {
							let mut name = parent_layout_name.into_value().clone();
							name.push_name(*field_name);

							return Some(Meta(name, metadata));
						}
					}
					LayoutConnection::Item => {
						let mut name = parent_layout_name.into_value().clone();
						name.push_name(&Name::new("item").unwrap());

						return Some(Meta(name, metadata));
					}
					_ => (),
				}
			}
		}

		None
	}

	fn simplify(
		&mut self,
		id: Id,
		metadata: M,
	) -> Result<(), Error<M>> {
		let Meta(desc, desc_metadata) = self.desc.clone()
			.try_unwrap()
			.map_err(|Conflict(Meta(desc1, meta), desc2)| {
				Meta(
					error::LayoutDescriptionMismatch {
						id,
						desc1,
						desc2
					}.into(),
					meta
				)
			})?
			.ok_or_else(|| {
				Meta(
					error::LayoutDescriptionMissing(id).into(),
					metadata.clone(),
				)
			})?;

		let simple_desc = match desc {
			Description::Simple(s) => s,
			Description::Intersection(_) => {
				todo!()
			}
		};

		self.simple_desc = Some(Meta(simple_desc, desc_metadata));
		Ok(())
	}

	fn build(
		&self,
		context: &Context<M>,
		as_resource: &resource::Data<M>,
		as_component: &component::Data<M>,
		metadata: M,
	) -> Result<treeldr::layout::Definition<M>, Error<M>> {
		let ty = self.ty.into_type_at_node_binding(context, as_resource.id, node::property::Layout::For)?;

		let name = as_component.name.try_unwrap().map_err(|e| e.at_functional_node_property(as_resource.id, node::property::Resource::Name))?;

		let restrictions_id = self.restrictions.try_unwrap().map_err(|e| e.at_functional_node_property(as_resource.id, node::property::Layout::WithRestrictions))?;

		let restrictions = restrictions_id.as_ref().map(|Meta(list_id, meta)| {
			let mut restrictions = Restrictions::default();
			let list = context.require_list(*list_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::WithRestrictions, meta.clone()))?;
			for restriction_value in list.iter(context) {
				let Meta(restriction_value, meta) = restriction_value?;
				let restriction_id = restriction_value.as_required_id(meta)?;
				let restriction_definition = context.require_layout_restriction(restriction_id).map_err(|e| e.at(meta.clone()))?;
				let restriction = restriction_definition.build()?;
				restrictions.insert(restriction)?
			}

			Ok(restrictions)
		}).transpose()?.unwrap_or_default();

		let Meta(desc, desc_metadata) = self.simple_desc.unwrap();
		let desc = match desc {
			SimpleDescription::Never => treeldr::layout::Description::Never(name),
			SimpleDescription::Primitive(n) => treeldr::layout::Description::Primitive(
				n.build(as_resource.id, restrictions.into_primitive(), &desc_metadata)?,
				name,
			),
			SimpleDescription::Reference(layout_id) => {
				let layout_ref = **context.require_layout(layout_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::Reference, desc_metadata.clone()))?;
				let r = treeldr::layout::Reference::new(name, layout_ref);
				treeldr::layout::Description::Reference(r)
			}
			SimpleDescription::Struct(fields_id) => {
				let name = name.ok_or_else(|| Meta(
					NodeBindingMissing {
						id: as_resource.id,
						property: node::property::Resource::Name.into()
					}.into(),
					metadata.clone()
				))?;

				let fields = context
					.require_list(fields_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::Fields, desc_metadata.clone()))?
					.iter(context)
					.map(|item| {
						let Meta(object, metadata) = item?.cloned();
						let field_id = object.into_required_id(&metadata)?;

						let field = context.require_layout_field(field_id).map_err(|e| e.at(metadata))?;
						let node = context.get(field_id).unwrap();
						let label = node.label().map(String::from);
						let doc = node.documentation().clone();
						field.build(field_id, label, doc, context)
					})
					.try_collect()?;

				let strct = treeldr::layout::Struct::new(name, fields);
				treeldr::layout::Description::Struct(strct)
			}
			SimpleDescription::Enum(options_id) => {
				let name = name.ok_or_else(|| Meta(
					NodeBindingMissing {
						id: as_resource.id,
						property: node::property::Resource::Name.into()
					}.into(),
					metadata.clone()
				))?;

				let variants: Vec<_> = context
					.require_list(options_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::Variants, desc_metadata.clone()))?
					.iter(context)
					.map(|item| {
						let Meta(object, variant_causes) = item?.cloned();
						let variant_id = object.into_required_id(&variant_causes)?;

						let variant = context.require_layout_variant(variant_id).map_err(|e| e.at(variant_causes.clone()))?;
						let node = context.get(variant_id).unwrap();
						let label = node.label().map(String::from);
						let doc = node.documentation().clone();
						Ok(Meta(variant.build(variant_id, label, doc, context)?, variant_causes))
					})
					.try_collect()?;

				let enm = treeldr::layout::Enum::new(name, variants);
				treeldr::layout::Description::Enum(enm)
			}
			SimpleDescription::Required(item_layout_id) => {
				let item_layout_ref = **context.require_layout(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::Required, desc_metadata.clone()))?;
				treeldr::layout::Description::Required(
					treeldr::layout::Required::new(name, item_layout_ref),
				)
			}
			SimpleDescription::Option(item_layout_id) => {
				let item_layout_ref = **context.require_layout(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::Option, desc_metadata.clone()))?;
				treeldr::layout::Description::Option(
					treeldr::layout::Optional::new(name, item_layout_ref),
				)
			}
			SimpleDescription::Set(item_layout_id) => {
				let item_layout_ref = **context.require_layout(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::Set, desc_metadata.clone()))?;
				treeldr::layout::Description::Set(
					treeldr::layout::Set::new(name, item_layout_ref, restrictions.into_container()),
				)
			}
			SimpleDescription::OneOrMany(item_layout_id) => {
				let item_layout_ref = **context.require_layout(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::OneOrMany, desc_metadata.clone()))?;
				treeldr::layout::Description::OneOrMany(
					treeldr::layout::OneOrMany::new(
						name,
						item_layout_ref,
						restrictions.into_container(),
					),
				)
			}
			SimpleDescription::Array(item_layout_id) => {
				let item_layout_ref = **context.require_layout(item_layout_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::Array, desc_metadata.clone()))?;
				let semantics = self.array_semantics.build(context, as_resource.id)?;
				treeldr::layout::Description::Array(
					treeldr::layout::Array::new(
						name,
						item_layout_ref,
						restrictions.into_container(),
						semantics
					)
				)
			},
			SimpleDescription::Alias(alias_layout_id) => {
				let name = name.ok_or_else(|| Meta(
					NodeBindingMissing {
						id: as_resource.id,
						property: node::property::Resource::Name.into()
					}.into(),
					metadata.clone()
				))?;

				let alias_layout_ref = **context.require_layout(alias_layout_id).map_err(|e| e.at_node_property(as_resource.id, node::property::Layout::Alias, desc_metadata.clone()))?;
				treeldr::layout::Description::Alias(name, alias_layout_ref)
			}
		};

		Ok(treeldr::layout::Definition::new(
			as_resource.id, ty, Meta(desc, desc_metadata)
		))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	For,
	Name,
	Description(DescriptionProperty),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptionProperty {
	Never,
	Primitive,
	Struct,
	Reference,
	Enum,
	Required,
	Option,
	Set,
	OneOrMany,
	Array,
	Alias,
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
