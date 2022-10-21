use crate::{error, utils::TryCollect, Context, Descriptions, Error, ObjectToId};
use locspan::Meta;
use rdf_types::IriVocabulary;
use treeldr::{metadata::Merge, Id, IriIndex, MetaOption, Name};

pub mod array;
pub mod field;
pub mod primitive;
pub mod variant;

pub use array::Array;
pub use primitive::{Primitive, Restricted as RestrictedPrimitive};

/// Layout description.
#[derive(Clone, Debug)]
pub enum Description<M> {
	Never,
	Primitive(RestrictedPrimitive<M>),
	Struct(Id),
	Reference(Id),
	Enum(Id),
	Required(Id),
	Option(Id),
	Set(Id),
	Array(Array<M>),
	Alias(Id),
}

/// Layout kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
	Never,
	Primitive(Option<Primitive>),
	Reference,
	Literal,
	Struct,
	Enum,
	Required,
	Option,
	Set,
	Array,
	Alias,
}

impl<M> Description<M> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Never => Kind::Never,
			Self::Reference(_) => Kind::Reference,
			Self::Struct(_) => Kind::Struct,
			Self::Primitive(n) => Kind::Primitive(n.primitive().value().cloned()),
			Self::Enum(_) => Kind::Enum,
			Self::Required(_) => Kind::Required,
			Self::Option(_) => Kind::Option,
			Self::Set(_) => Kind::Set,
			Self::Array(_) => Kind::Array,
			Self::Alias(_) => Kind::Alias,
		}
	}

	pub fn sub_layouts<D: Descriptions<M>>(
		&self,
		context: &Context<M, D>,
		metadata: &M,
	) -> Result<Vec<SubLayout<M>>, Error<M>>
	where
		M: Clone,
	{
		let mut sub_layouts = Vec::new();

		match self {
			Description::Struct(fields_id) => {
				let fields = context.require_list(*fields_id, metadata)?.iter(context);
				for item in fields {
					let Meta(object, metadata) = item?.clone();
					let field_id = object.into_id(&metadata)?;
					let field = context.require_layout_field(field_id, &metadata)?;
					let field_layout_id = field.require_layout(field.metadata())?;

					sub_layouts.push(SubLayout {
						layout: field_layout_id.clone(),
						connection: LayoutConnection::FieldContainer(field_id),
					});

					let field_layout =
						context.require_layout(**field_layout_id, field_layout_id.metadata())?;
					if let Some(container_desc) = field_layout.description() {
						match container_desc.as_standard() {
							Some(standard_desc) => {
								let item_layout_id = match standard_desc {
									Description::Required(id) => *id,
									Description::Option(id) => *id,
									Description::Set(id) => *id,
									Description::Array(a) => a.item_layout(),
									_ => panic!("invalid field layout: not a container"),
								};

								sub_layouts.push(SubLayout {
									layout: Meta(item_layout_id, container_desc.metadata().clone()),
									connection: LayoutConnection::FieldItem(field_id),
								});
							}
							None => {
								panic!("invalid field layout: not a container")
							}
						}
					}
				}
			}
			Description::Set(item_layout_id) => sub_layouts.push(SubLayout {
				layout: Meta(*item_layout_id, metadata.clone()),
				connection: LayoutConnection::Item,
			}),
			Description::Array(array) => sub_layouts.push(SubLayout {
				layout: Meta(array.item_layout(), metadata.clone()),
				connection: LayoutConnection::Item,
			}),
			_ => (),
		}

		Ok(sub_layouts)
	}

	pub fn dependencies(
		&self,
		_id: Id,
		_nodes: &super::context::allocated::Nodes<M>,
		_causes: &M,
	) -> Result<Vec<crate::Item<M>>, Error<M>> {
		Ok(Vec::new())
	}

	pub fn build(
		self,
		id: Id,
		name: MetaOption<Name, M>,
		nodes: &mut super::context::allocated::Nodes<M>,
		metadata: &M,
	) -> Result<treeldr::layout::Description<M>, Error<M>>
	where
		M: Clone,
	{
		use field::Build as BuildField;
		use variant::Build as BuildVariant;

		fn require_name<M>(
			id: Id,
			name: MetaOption<Name, M>,
			metadata: &M,
		) -> Result<Meta<Name, M>, Error<M>>
		where
			M: Clone,
		{
			name.ok_or_else(|| Meta(error::LayoutMissingName(id).into(), metadata.clone()))
		}

		match self {
			Description::Never => Ok(treeldr::layout::Description::Never(name)),
			Description::Primitive(n) => Ok(treeldr::layout::Description::Primitive(
				n.build(id, metadata)?,
				name,
			)),
			Description::Reference(layout_id) => {
				let layout_ref = **nodes.require_layout(layout_id, metadata)?;
				let r = treeldr::layout::Reference::new(name, layout_ref);
				Ok(treeldr::layout::Description::Reference(r))
			}
			Description::Struct(fields_id) => {
				let name = require_name(id, name, metadata)?;
				let fields = nodes
					.require_list(fields_id, metadata)?
					.iter(nodes)
					.map(|item| {
						let Meta(object, metadata) = item?.clone();
						let field_id = object.into_id(&metadata)?;

						let field = nodes.require_layout_field(field_id, &metadata)?;
						let node = nodes.get(field_id).unwrap();
						let label = node.label().map(String::from);
						let doc = node.documentation().clone();
						field.build(label, doc, nodes)
					})
					.try_collect()?;

				let strct = treeldr::layout::Struct::new(name, fields);
				Ok(treeldr::layout::Description::Struct(strct))
			}
			Description::Enum(options_id) => {
				let name = require_name(id, name, metadata)?;

				let variants: Vec<_> = nodes
					.require_list(options_id, metadata)?
					.iter(nodes)
					.map(|item| {
						let Meta(object, variant_causes) = item?.clone();
						let variant_id = object.into_id(&variant_causes)?;

						let variant = nodes.require_layout_variant(variant_id, &variant_causes)?;
						let node = nodes.get(variant_id).unwrap();
						let label = node.label().map(String::from);
						let doc = node.documentation().clone();
						Ok(Meta(variant.build(label, doc, nodes)?, variant_causes))
					})
					.try_collect()?;

				let enm = treeldr::layout::Enum::new(name, variants);
				Ok(treeldr::layout::Description::Enum(enm))
			}
			Description::Required(item_layout_id) => {
				let item_layout_ref = **nodes.require_layout(item_layout_id, metadata)?;
				Ok(treeldr::layout::Description::Required(
					treeldr::layout::Required::new(name, item_layout_ref),
				))
			}
			Description::Option(item_layout_id) => {
				let item_layout_ref = **nodes.require_layout(item_layout_id, metadata)?;
				Ok(treeldr::layout::Description::Option(
					treeldr::layout::Optional::new(name, item_layout_ref),
				))
			}
			Description::Set(item_layout_id) => {
				let item_layout_ref = **nodes.require_layout(item_layout_id, metadata)?;
				Ok(treeldr::layout::Description::Set(
					treeldr::layout::Set::new(name, item_layout_ref),
				))
			}
			Description::Array(array) => Ok(treeldr::layout::Description::Array(
				array.build(name, nodes, metadata)?,
			)),
			Description::Alias(alias_layout_id) => {
				let name = require_name(id, name, metadata)?;

				let alias_layout_ref = **nodes.require_layout(alias_layout_id, metadata)?;
				Ok(treeldr::layout::Description::Alias(name, alias_layout_ref))
			}
		}
	}
}

pub trait PseudoDescription<M>: Clone + From<Description<M>> {
	fn as_standard(&self) -> Option<&Description<M>>;

	fn as_standard_mut(&mut self) -> Option<&mut Description<M>>;

	fn try_unify(
		self,
		other: Self,
		id: Id,
		metadata: M,
		other_causes: M,
	) -> Result<Meta<Self, M>, Error<M>>
	where
		M: Merge;
}

impl<M: Clone> PseudoDescription<M> for Description<M> {
	fn as_standard(&self) -> Option<&Description<M>> {
		Some(self)
	}

	fn as_standard_mut(&mut self) -> Option<&mut Description<M>> {
		Some(self)
	}

	fn try_unify(
		self,
		other: Self,
		id: Id,
		meta: M,
		other_meta: M,
	) -> Result<Meta<Self, M>, Error<M>>
	where
		M: Merge,
	{
		match (self, other) {
			(Self::Never, Self::Never) => Ok(Meta(Self::Never, meta.merged_with(other_meta))),
			(Self::Primitive(a), Self::Primitive(b)) => Ok(Meta(
				Self::Primitive(a.try_unify(id, b)?),
				meta.merged_with(other_meta),
			)),
			(Self::Struct(a), Self::Struct(b)) if a == b => {
				Ok(Meta(Self::Struct(a), meta.merged_with(other_meta)))
			}
			(Self::Reference(a), Self::Reference(b)) if a == b => {
				Ok(Meta(Self::Reference(a), meta.merged_with(other_meta)))
			}
			(Self::Enum(a), Self::Enum(b)) if a == b => {
				Ok(Meta(Self::Enum(a), meta.merged_with(other_meta)))
			}
			(Self::Set(a), Self::Set(b)) if a == b => {
				Ok(Meta(Self::Set(a), meta.merged_with(other_meta)))
			}
			(Self::Array(a), Self::Array(b)) => {
				let Meta(result, meta) = a.try_unify(b, id, meta, other_meta)?;
				Ok(Meta(Self::Array(result), meta))
			}
			(Self::Alias(a), Self::Alias(b)) if a == b => {
				Ok(Meta(Self::Alias(a), meta.merged_with(other_meta)))
			}
			_ => Err(Error::new(
				error::LayoutMismatchDescription { id, because: meta }.into(),
				other_meta,
			)),
		}
	}
}

/// Layout definition.
#[derive(Clone)]
pub struct Definition<M, D = Description<M>> {
	/// Identifier of the layout.
	id: Id,

	/// Optional name.
	///
	/// If not provided, the name is generated using the `default_name`
	/// method. If it conflicts with another name or failed to be generated,
	/// then a name must be explicitly defined by the user.
	name: MetaOption<Name, M>,

	/// Type for which this layout is defined.
	ty: MetaOption<Id, M>,

	/// Layout description.
	desc: MetaOption<D, M>,
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

impl<M, D> Definition<M, D> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			name: MetaOption::default(),
			ty: MetaOption::default(),
			desc: MetaOption::default(),
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<&Meta<Id, M>> {
		self.ty.as_ref()
	}

	pub fn name(&self) -> Option<&Meta<Name, M>> {
		self.name.as_ref()
	}

	pub fn set_name(&mut self, name: Name, metadata: M) -> Result<(), Error<M>> {
		self.name.try_set(
			name,
			metadata,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::LayoutMismatchName {
						id: self.id,
						expected,
						found,
						because: expected_meta,
					}
					.into(),
					found_meta,
				)
			},
		)
	}

	pub fn description(&self) -> Option<&Meta<D, M>> {
		self.desc.as_ref()
	}

	pub fn description_mut(&mut self) -> Option<&mut Meta<D, M>> {
		self.desc.as_mut()
	}

	/// Declare the type for which this layout is defined.
	pub fn set_type(&mut self, ty_ref: Id, metadata: M) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.ty.try_set(
			ty_ref,
			metadata,
			|Meta(expected, expected_meta), Meta(found, found_meta)| {
				Error::new(
					error::LayoutMismatchType {
						id: self.id,
						expected,
						found,
						because: expected_meta,
					}
					.into(),
					found_meta,
				)
			},
		)
	}

	pub fn set_description(&mut self, desc: D, metadata: M) -> Result<(), Error<M>>
	where
		M: Merge,
		D: PseudoDescription<M>,
	{
		self.desc.try_unify(
			desc,
			metadata,
			|Meta(current_desc, current_meta), Meta(desc, meta)| {
				current_desc.try_unify(desc, self.id, current_meta, meta)
			},
		)
	}

	pub fn replace_description(&mut self, desc: MetaOption<D, M>) {
		self.desc = desc
	}

	pub fn try_map<U, E>(self, f: impl FnOnce(D) -> Result<U, E>) -> Result<Definition<M, U>, E> {
		Ok(Definition {
			id: self.id,
			name: self.name,
			ty: self.ty,
			desc: self.desc.try_map(f)?,
		})
	}
}

impl<M: Merge, D: PseudoDescription<M>> Definition<M, D> {
	pub fn set_primitive(
		&mut self,
		primitive: RestrictedPrimitive<M>,
		metadata: M,
	) -> Result<(), Error<M>> {
		self.set_description(Description::Primitive(primitive).into(), metadata)
	}

	pub fn set_alias(&mut self, alias: Id, metadata: M) -> Result<(), Error<M>> {
		self.set_description(Description::Alias(alias).into(), metadata)
	}

	pub fn set_fields(&mut self, fields: Id, metadata: M) -> Result<(), Error<M>> {
		self.set_description(Description::Struct(fields).into(), metadata)
	}

	pub fn set_reference(&mut self, id_layout: Id, metadata: M) -> Result<(), Error<M>> {
		self.set_description(Description::Reference(id_layout).into(), metadata)
	}

	pub fn set_enum(&mut self, items: Id, metadata: M) -> Result<(), Error<M>> {
		self.set_description(Description::Enum(items).into(), metadata)
	}

	pub fn set_required(&mut self, item: Id, metadata: M) -> Result<(), Error<M>> {
		self.set_description(Description::Required(item).into(), metadata)
	}

	pub fn set_option(&mut self, item: Id, metadata: M) -> Result<(), Error<M>> {
		self.set_description(Description::Option(item).into(), metadata)
	}

	pub fn set_set(&mut self, item: Id, metadata: M) -> Result<(), Error<M>> {
		self.set_description(Description::Set(item).into(), metadata)
	}

	pub fn set_array(
		&mut self,
		item: Id,
		semantics: Option<array::Semantics<M>>,
		metadata: M,
	) -> Result<(), Error<M>> {
		self.set_description(
			Description::Array(Array::new(item, semantics)).into(),
			metadata,
		)
	}
}

impl<M: Clone, D: PseudoDescription<M>> Definition<M, D> {
	pub fn set_array_list_first(&mut self, first_prop: Id, metadata: M) -> Result<(), Error<M>> {
		let id = self.id;
		match self.description_mut() {
			Some(desc) => match desc.0.as_standard_mut() {
				Some(Description::Array(array)) => array.set_list_first(id, first_prop, metadata),
				_ => Err(Error::new(
					error::LayoutMismatchDescription {
						id,
						because: desc.1.clone(),
					}
					.into(),
					metadata,
				)),
			},
			None => todo!("error: not a list"),
		}
	}

	pub fn set_array_list_rest(&mut self, value: Id, metadata: M) -> Result<(), Error<M>> {
		let id = self.id;
		match self.description_mut() {
			Some(desc) => match desc.0.as_standard_mut() {
				Some(Description::Array(array)) => array.set_list_rest(id, value, metadata),
				_ => Err(Error::new(
					error::LayoutMismatchDescription {
						id,
						because: desc.1.clone(),
					}
					.into(),
					metadata,
				)),
			},
			None => todo!("error: not a list"),
		}
	}

	pub fn set_array_list_nil(&mut self, value: Id, metadata: M) -> Result<(), Error<M>> {
		let id = self.id;
		match self.description_mut() {
			Some(desc) => match desc.0.as_standard_mut() {
				Some(Description::Array(array)) => array.set_list_nil(id, value, metadata),
				_ => Err(Error::new(
					error::LayoutMismatchDescription {
						id,
						because: desc.1.clone(),
					}
					.into(),
					metadata,
				)),
			},
			None => todo!("error: not a list"),
		}
	}
}

impl<M: Clone> Definition<M> {
	pub fn sub_layouts(&self, context: &Context<M>) -> Result<Vec<SubLayout<M>>, Error<M>> {
		match self.desc.as_ref() {
			Some(desc) => desc.sub_layouts(context, desc.metadata()),
			None => Ok(Vec::new()),
		}
	}

	pub fn dependencies(
		&self,
		nodes: &super::context::allocated::Nodes<M>,
		_causes: &M,
	) -> Result<Vec<crate::Item<M>>, Error<M>> {
		match self.desc.as_ref() {
			Some(desc) => desc.dependencies(self.id, nodes, desc.metadata()),
			None => Ok(Vec::new()),
		}
	}

	/// Build a default name for this layout.
	pub fn default_name<C: Descriptions<M>>(
		&self,
		context: &Context<M, C>,
		vocabulary: &impl IriVocabulary<Iri = IriIndex>,
		parent_layouts: &[Meta<ParentLayout, M>],
		metadata: M,
	) -> Result<Option<Meta<Name, M>>, Error<M>> {
		if let Id::Iri(term) = self.id {
			if let Ok(Some(name)) = Name::from_iri(vocabulary.iri(&term).unwrap()) {
				return Ok(Some(Meta(name, metadata)));
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
			let parent_layout = context.require_layout(parent.layout, &metadata)?.value();

			if let Some(parent_layout_name) = parent_layout.name() {
				match parent.connection {
					LayoutConnection::FieldItem(field_id) => {
						let field = context.require_layout_field(field_id, &metadata)?.value();

						if let Some(field_name) = field.name() {
							let mut name = parent_layout_name.value().clone();
							name.push_name(field_name);

							return Ok(Some(Meta(name, metadata)));
						}
					}
					LayoutConnection::Item => {
						let mut name = parent_layout_name.value().clone();
						name.push_name(&Name::new("item").unwrap());

						return Ok(Some(Meta(name, metadata)));
					}
					_ => (),
				}
			}
		}

		Ok(None)
	}
}

impl<M: Clone> crate::Build<M> for Definition<M> {
	type Target = treeldr::layout::Definition<M>;

	fn build(
		self,
		nodes: &mut super::context::allocated::Nodes<M>,
		_dependencies: crate::Dependencies<M>,
		metadata: M,
	) -> Result<Self::Target, Error<M>> {
		let ty = self.ty.try_map_with_causes(|Meta(ty_id, metadata)| {
			Ok(Meta(**nodes.require_type(ty_id, &metadata)?, metadata))
		})?;

		let Meta(desc, desc_metadata) = self.desc.ok_or_else(|| {
			Meta(
				error::LayoutMissingDescription(self.id).into(),
				metadata.clone(),
			)
		})?;

		let desc = Meta(
			desc.build(self.id, self.name, nodes, &desc_metadata)?,
			desc_metadata,
		);

		Ok(treeldr::layout::Definition::new(
			self.id, ty, desc, metadata,
		))
	}
}
