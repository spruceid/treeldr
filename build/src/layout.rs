use crate::{error, utils::TryCollect, Context, Descriptions, Error, ObjectToId};
use locspan::Location;
use treeldr::{Caused, Causes, Id, MaybeSet, Name, Vocabulary, WithCauses};

pub mod array;
pub mod enumeration;
pub mod field;
pub mod literal;
pub mod structure;
pub mod variant;

pub use array::Array;
pub use treeldr::layout::{literal::RegExp, Primitive};

#[derive(Clone, Debug)]
pub enum Description<F> {
	Never,
	Primitive(Primitive),
	Struct(Id),
	Reference(Id),
	Literal(RegExp),
	Enum(Id),
	Set(Id),
	Array(Array<F>),
	Alias(Id),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
	Never,
	Primitive(Primitive),
	Reference,
	Literal,
	Struct,
	Enum,
	Set,
	Array,
	Alias,
}

impl<F> Description<F> {
	pub fn ty(&self) -> Type {
		match self {
			Self::Never => Type::Never,
			Self::Reference(_) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Primitive(n) => Type::Primitive(*n),
			Self::Literal(_) => Type::Literal,
			Self::Enum(_) => Type::Enum,
			Self::Set(_) => Type::Set,
			Self::Array(_) => Type::Array,
			Self::Alias(_) => Type::Alias,
		}
	}

	pub fn sub_layouts<D: Descriptions<F>>(
		&self,
		context: &Context<F, D>,
		causes: &Causes<F>,
	) -> Result<Vec<SubLayout<F>>, Error<F>>
	where
		F: Clone + Ord,
	{
		let mut sub_layouts = Vec::new();

		match self {
			Description::Struct(fields_id) => {
				let fields = context
					.require_list(*fields_id, causes.preferred().cloned())?
					.iter(context);
				for item in fields {
					let (object, causes) = item?.clone().into_parts();
					let field_id = object.into_id(causes.preferred())?;
					let field = context.require_layout_field(field_id, causes.into_preferred())?;
					let field_layout_id = field.require_layout(field.causes())?;

					sub_layouts.push(SubLayout {
						layout: field_layout_id.clone(),
						connection: LayoutConnection::Field(field_id),
					});
				}
			}
			Description::Set(item_layout_id) => sub_layouts.push(SubLayout {
				layout: WithCauses::new(*item_layout_id, causes.clone()),
				connection: LayoutConnection::Item,
			}),
			Description::Array(array) => sub_layouts.push(SubLayout {
				layout: WithCauses::new(array.item_layout(), causes.clone()),
				connection: LayoutConnection::Item,
			}),
			_ => (),
		}

		Ok(sub_layouts)
	}

	pub fn dependencies(
		&self,
		_id: Id,
		_nodes: &super::context::allocated::Nodes<F>,
		_causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>> {
		// match self {
		// 	Description::Struct(field_list_id) => {
		// 		let mut dependencies = Vec::new();
		// 		let field_list = nodes.require_list(*field_list_id, causes.preferred().cloned())?;

		// 		for item in field_list.iter(nodes) {
		// 			let (object, field_causes) = item?.clone().into_parts();
		// 			let field_id = object.into_id(field_causes.preferred())?;

		// 			let field =
		// 				nodes.require_layout_field(field_id, field_causes.preferred().cloned())?;

		// 			if let Some(prop_id) = field.property() {
		// 				let prop_ref = **nodes
		// 					.require_property(**prop_id, prop_id.causes().preferred().cloned())?;

		// 				dependencies.push(crate::Item::Property(prop_ref));
		// 			}

		// 			if let Some(layout_id) = field.layout() {
		// 				let layout_ref = **nodes
		// 					.require_layout(**layout_id, layout_id.causes().preferred().cloned())?;

		// 				dependencies.push(crate::Item::Layout(layout_ref));
		// 			}
		// 		}

		// 		Ok(dependencies)
		// 	}
		// 	Description::Enum(variant_list_id) => {
		// 		let layouts =
		// 			nodes
		// 				.require_list(*variant_list_id, causes.preferred().cloned())?
		// 				.iter(nodes)
		// 				.map(|item| {
		// 					let (object, variant_causes) = item?.clone().into_parts();
		// 					let variant_id = object.into_id(variant_causes.preferred())?;

		// 					let variant = nodes.require_layout_variant(
		// 						variant_id,
		// 						variant_causes.preferred().cloned(),
		// 					)?;

		// 					let layout = variant.layout().clone().try_map_with_causes(
		// 						|layout_id, causes| {
		// 							Ok(*nodes
		// 								.require_layout(layout_id, causes.preferred().cloned())?
		// 								.inner())
		// 						},
		// 					)?;

		// 					Ok(layout.into_value().map(crate::Item::Layout))
		// 				})
		// 				.try_filter_collect()?;
		// 		Ok(layouts)
		// 	}
		// 	_ => Ok(Vec::new()),
		// }
		Ok(Vec::new())
	}

	pub fn build(
		self,
		id: Id,
		name: MaybeSet<Name, F>,
		nodes: &mut super::context::allocated::Nodes<F>,
		causes: &Causes<F>,
	) -> Result<treeldr::layout::Description<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		use field::Build as BuildField;
		use variant::Build as BuildVariant;

		fn require_name<F>(
			id: Id,
			name: MaybeSet<Name, F>,
			causes: &Causes<F>,
		) -> Result<WithCauses<Name, F>, Error<F>>
		where
			F: Clone,
		{
			name.ok_or_else(|| {
				Caused::new(
					error::LayoutMissingName(id).into(),
					causes.preferred().cloned(),
				)
			})
		}

		match self {
			Description::Never => Ok(treeldr::layout::Description::Never(name)),
			Description::Primitive(n) => Ok(treeldr::layout::Description::Primitive(n, name)),
			Description::Reference(layout_id) => {
				let layout_ref = *nodes
					.require_layout(layout_id, causes.preferred().cloned())?
					.inner();
				Ok(treeldr::layout::Description::Reference(layout_ref, name))
			}
			Description::Struct(fields_id) => {
				let name = require_name(id, name, causes)?;
				let fields = nodes
					.require_list(fields_id, causes.preferred().cloned())?
					.iter(nodes)
					.map(|item| {
						let (object, causes) = item?.clone().into_parts();
						let field_id = object.into_id(causes.preferred())?;

						let field =
							nodes.require_layout_field(field_id, causes.into_preferred())?;
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
				let name = require_name(id, name, causes)?;

				let variants: Vec<_> = nodes
					.require_list(options_id, causes.preferred().cloned())?
					.iter(nodes)
					.map(|item| {
						let (object, variant_causes) = item?.clone().into_parts();
						let variant_id = object.into_id(variant_causes.preferred())?;

						let variant = nodes.require_layout_variant(
							variant_id,
							variant_causes.preferred().cloned(),
						)?;
						let node = nodes.get(variant_id).unwrap();
						let label = node.label().map(String::from);
						let doc = node.documentation().clone();
						Ok(WithCauses::new(
							variant.build(label, doc, nodes)?,
							variant_causes,
						))
					})
					.try_collect()?;

				let enm = treeldr::layout::Enum::new(name, variants);
				Ok(treeldr::layout::Description::Enum(enm))
			}
			Description::Literal(regexp) => {
				let name = require_name(id, name, causes)?;
				let lit = treeldr::layout::Literal::new(regexp, name, id.is_blank());
				Ok(treeldr::layout::Description::Literal(lit))
			}
			Description::Set(item_layout_id) => {
				let item_layout_ref = *nodes
					.require_layout(item_layout_id, causes.preferred().cloned())?
					.inner();
				Ok(treeldr::layout::Description::Set(
					treeldr::layout::Set::new(name, item_layout_ref),
				))
			}
			Description::Array(array) => Ok(treeldr::layout::Description::Array(
				array.build(name, nodes, causes)?,
			)),
			Description::Alias(alias_layout_id) => {
				let name = require_name(id, name, causes)?;

				let alias_layout_ref = *nodes
					.require_layout(alias_layout_id, causes.preferred().cloned())?
					.inner();
				Ok(treeldr::layout::Description::Alias(name, alias_layout_ref))
			}
		}
	}
}

pub trait PseudoDescription<F>: Clone + From<Description<F>> {
	fn as_standard(&self) -> Option<&Description<F>>;

	fn as_standard_mut(&mut self) -> Option<&mut Description<F>>;

	fn try_unify(
		self,
		other: Self,
		id: Id,
		causes: &Causes<F>,
		other_causes: &Causes<F>,
	) -> Result<Self, Error<F>>;
}

impl<F: Clone + Ord> PseudoDescription<F> for Description<F> {
	fn as_standard(&self) -> Option<&Description<F>> {
		Some(self)
	}

	fn as_standard_mut(&mut self) -> Option<&mut Description<F>> {
		Some(self)
	}

	fn try_unify(
		self,
		other: Self,
		id: Id,
		causes: &Causes<F>,
		other_causes: &Causes<F>,
	) -> Result<Self, Error<F>> {
		match (self, other) {
			(Self::Never, Self::Never) => Ok(Self::Never),
			(Self::Primitive(a), Self::Primitive(b)) if a == b => Ok(Self::Primitive(a)),
			(Self::Struct(a), Self::Struct(b)) if a == b => Ok(Self::Struct(a)),
			(Self::Reference(a), Self::Reference(b)) if a == b => Ok(Self::Reference(a)),
			(Self::Literal(a), Self::Literal(b)) if a == b => Ok(Self::Literal(a)),
			(Self::Enum(a), Self::Enum(b)) if a == b => Ok(Self::Enum(a)),
			(Self::Set(a), Self::Set(b)) if a == b => Ok(Self::Set(a)),
			(Self::Array(a), Self::Array(b)) => {
				Ok(Self::Array(a.try_unify(b, id, causes, other_causes)?))
			}
			(Self::Alias(a), Self::Alias(b)) if a == b => Ok(Self::Alias(a)),
			_ => Err(Error::new(
				error::LayoutMismatchDescription {
					id,
					because: causes.preferred().cloned(),
				}
				.into(),
				other_causes.preferred().cloned(),
			)),
		}
	}
}

/// Layout definition.
#[derive(Clone)]
pub struct Definition<F, D = Description<F>> {
	/// Identifier of the layout.
	id: Id,

	/// Optional name.
	///
	/// If not provided, the name is generated using the `default_name`
	/// method. If it conflicts with another name or failed to be generated,
	/// then a name must be explicitly defined by the user.
	name: MaybeSet<Name, F>,

	/// Type for which this layout is defined.
	ty: MaybeSet<Id, F>,

	/// Layout description.
	desc: MaybeSet<D, F>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutConnection {
	Field(Id),
	Variant(Id),
	Item,
}

pub struct SubLayout<F> {
	pub connection: LayoutConnection,
	pub layout: WithCauses<Id, F>,
}

pub struct ParentLayout {
	pub connection: LayoutConnection,
	pub layout: Id,
}

impl<F, D> Definition<F, D> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			name: MaybeSet::default(),
			ty: MaybeSet::default(),
			desc: MaybeSet::default(),
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<&WithCauses<Id, F>> {
		self.ty.with_causes()
	}

	pub fn name(&self) -> Option<&WithCauses<Name, F>> {
		self.name.with_causes()
	}

	pub fn set_name(&mut self, name: Name, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.name
			.try_set(name, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutMismatchName {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}

	pub fn description(&self) -> Option<&WithCauses<D, F>> {
		self.desc.with_causes()
	}

	pub fn description_mut(&mut self) -> Option<&mut WithCauses<D, F>> {
		self.desc.with_causes_mut()
	}

	/// Declare the type for which this layout is defined.
	pub fn set_type(&mut self, ty_ref: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.ty
			.try_set(ty_ref, cause, |expected, found, because, causes| {
				Error::new(
					error::LayoutMismatchType {
						id: self.id,
						expected,
						found,
						because: because.preferred().cloned(),
					}
					.into(),
					causes.preferred().cloned(),
				)
			})
	}

	pub fn set_description(&mut self, desc: D, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
		D: PseudoDescription<F>,
	{
		self.desc
			.try_unify(desc, cause, |current_desc, desc, causes, cause| {
				current_desc.try_unify(desc, self.id, causes, cause)
			})
	}

	pub fn replace_description(&mut self, desc: MaybeSet<D, F>) {
		self.desc = desc
	}

	pub fn try_map<U, E>(self, f: impl FnOnce(D) -> Result<U, E>) -> Result<Definition<F, U>, E> {
		Ok(Definition {
			id: self.id,
			name: self.name,
			ty: self.ty,
			desc: self.desc.try_map(f)?,
		})
	}
}

impl<F: Clone + Ord, D: PseudoDescription<F>> Definition<F, D> {
	pub fn set_primitive(
		&mut self,
		primitive: Primitive,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>> {
		self.set_description(Description::Primitive(primitive).into(), cause)
	}

	pub fn set_fields(&mut self, fields: Id, cause: Option<Location<F>>) -> Result<(), Error<F>> {
		self.set_description(Description::Struct(fields).into(), cause)
	}

	pub fn set_deref_to(&mut self, target: Id, cause: Option<Location<F>>) -> Result<(), Error<F>> {
		self.set_description(Description::Reference(target).into(), cause)
	}

	pub fn set_literal(
		&mut self,
		regexp: RegExp,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>> {
		self.set_description(Description::Literal(regexp).into(), cause)
	}

	pub fn set_enum(&mut self, items: Id, cause: Option<Location<F>>) -> Result<(), Error<F>> {
		self.set_description(Description::Enum(items).into(), cause)
	}

	pub fn set_set(&mut self, item: Id, cause: Option<Location<F>>) -> Result<(), Error<F>> {
		self.set_description(Description::Set(item).into(), cause)
	}

	pub fn set_array(
		&mut self,
		item: Id,
		semantics: Option<array::Semantics<F>>,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>> {
		self.set_description(
			Description::Array(Array::new(item, semantics)).into(),
			cause,
		)
	}

	pub fn set_array_list_first(
		&mut self,
		first_prop: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>> {
		let id = self.id;
		match self.description_mut() {
			Some(desc) => match desc.inner_mut().as_standard_mut() {
				Some(Description::Array(array)) => array.set_list_first(id, first_prop, cause),
				_ => Err(Error::new(
					error::LayoutMismatchDescription {
						id,
						because: desc.causes().preferred().cloned(),
					}
					.into(),
					cause,
				)),
			},
			None => Err(Error::new(
				error::LayoutMismatchDescription { id, because: None }.into(),
				cause,
			)),
		}
	}

	pub fn set_array_list_rest(
		&mut self,
		value: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>> {
		let id = self.id;
		match self.description_mut() {
			Some(desc) => match desc.inner_mut().as_standard_mut() {
				Some(Description::Array(array)) => array.set_list_rest(id, value, cause),
				_ => Err(Error::new(
					error::LayoutMismatchDescription {
						id,
						because: desc.causes().preferred().cloned(),
					}
					.into(),
					cause,
				)),
			},
			None => Err(Error::new(
				error::LayoutMismatchDescription { id, because: None }.into(),
				cause,
			)),
		}
	}

	pub fn set_array_list_nil(
		&mut self,
		value: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>> {
		let id = self.id;
		match self.description_mut() {
			Some(desc) => match desc.inner_mut().as_standard_mut() {
				Some(Description::Array(array)) => array.set_list_nil(id, value, cause),
				_ => Err(Error::new(
					error::LayoutMismatchDescription {
						id,
						because: desc.causes().preferred().cloned(),
					}
					.into(),
					cause,
				)),
			},
			None => Err(Error::new(
				error::LayoutMismatchDescription { id, because: None }.into(),
				cause,
			)),
		}
	}
}

impl<F: Clone + Ord> Definition<F> {
	pub fn sub_layouts(&self, context: &Context<F>) -> Result<Vec<SubLayout<F>>, Error<F>> {
		match self.desc.with_causes() {
			Some(desc) => desc.sub_layouts(context, desc.causes()),
			None => Ok(Vec::new()),
		}
	}

	pub fn dependencies(
		&self,
		nodes: &super::context::allocated::Nodes<F>,
		_causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>> {
		match self.desc.with_causes() {
			Some(desc) => desc.dependencies(self.id, nodes, desc.causes()),
			None => Ok(Vec::new()),
		}
	}

	/// Build a default name for this layout.
	pub fn default_name<C: Descriptions<F>>(
		&self,
		context: &Context<F, C>,
		vocabulary: &Vocabulary,
		parent_layouts: &[WithCauses<ParentLayout, F>],
		cause: Option<Location<F>>,
	) -> Result<Option<Caused<Name, F>>, Error<F>> {
		if let Id::Iri(iri) = self.id {
			if let Some(name) = iri.iri(vocabulary).unwrap().path().file_name() {
				if let Ok(name) = Name::new(name) {
					return Ok(Some(Caused::new(name, cause)));
				}
			}
		}

		if let Some(Description::Literal(regexp)) = self.desc.value() {
			if let Some(singleton) = regexp.as_singleton() {
				if let Ok(singleton_name) = Name::new(singleton) {
					let mut name = Name::new("const").unwrap();
					name.push_name(&singleton_name);
					return Ok(Some(Caused::new(name, cause)));
				}
			}
		}

		if parent_layouts.len() == 1 {
			let parent = &parent_layouts[0];
			let parent_layout = context
				.require_layout(parent.layout, cause.clone())?
				.inner();

			if let Some(parent_layout_name) = parent_layout.name() {
				match parent.connection {
					LayoutConnection::Field(field_id) => {
						let field = context
							.require_layout_field(field_id, cause.clone())?
							.inner();

						if let Some(field_name) = field.name() {
							let mut name = parent_layout_name.inner().clone();
							name.push_name(field_name);

							return Ok(Some(Caused::new(name, cause)));
						}
					}
					LayoutConnection::Item => {
						let mut name = parent_layout_name.inner().clone();
						name.push_name(&Name::new("item").unwrap());

						return Ok(Some(Caused::new(name, cause)));
					}
					_ => (),
				}
			}
		}

		Ok(None)
	}
}

impl<F: Ord + Clone> crate::Build<F> for Definition<F> {
	type Target = treeldr::layout::Definition<F>;

	fn build(
		self,
		nodes: &mut super::context::allocated::Nodes<F>,
		_dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Error<F>> {
		let ty = self.ty.try_map_with_causes(|ty_id, causes| {
			Ok(**nodes.require_type(ty_id, causes.preferred().cloned())?)
		})?;

		let desc = self.desc.ok_or_else(|| {
			Caused::new(
				error::LayoutMissingDescription(self.id).into(),
				causes.preferred().cloned(),
			)
		})?;

		let desc = desc.try_map_with_causes(|desc, desc_causes| {
			desc.build(self.id, self.name, nodes, desc_causes)
		})?;

		Ok(treeldr::layout::Definition::new(self.id, ty, desc, causes))
	}
}
