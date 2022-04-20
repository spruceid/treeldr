use crate::{
	error,
	utils::{TryCollect, TryFilterCollect},
	Context, Descriptions, Error,
};
use locspan::Location;
use treeldr::{vocab, Caused, Causes, Id, MaybeSet, WithCauses};
use vocab::Name;

pub mod enumeration;
pub mod field;
pub mod literal;
pub mod structure;
pub mod variant;

pub use treeldr::layout::{literal::RegExp, Native};

/// Layout definition.
pub struct Definition<F, D = Description> {
	/// Identifier of the layout.
	id: Id,

	/// Optional name.
	///
	/// If not provided, the name is generated using the `default_name`
	/// method. If it conflicts with another name or failed to be generated,
	/// then a name must be explicitly defined by the user.
	name: MaybeSet<vocab::Name, F>,

	/// Type for which this layout is defined.
	ty: MaybeSet<Id, F>,

	/// Layout description.
	desc: MaybeSet<D, F>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutConnection {
	Field(Id),
	Variant(Id),
}

pub struct SubLayout<F> {
	pub connection: LayoutConnection,
	pub layout: WithCauses<Id, F>,
}

pub struct ParentLayout {
	pub connection: LayoutConnection,
	pub layout: Id,
}

pub trait PseudoDescription<F>: PartialEq + From<Description> {
	type Error: From<Error<F>>;

	fn as_standard(&self) -> Option<&Description>;

	fn sub_layouts<D: Descriptions<F>>(&self, context: &Context<F, D>, causes: &Causes<F>) -> Result<Vec<SubLayout<F>>, Self::Error>;

	fn dependencies(
		&self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Self::Error>;

	fn reduce(self, id: Id, name: &MaybeSet<vocab::Name, F>, ty: &MaybeSet<Id, F>) -> Result<Description, Self::Error>;
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Description {
	Native(Native),
	Struct(Id),
	Reference(Id),
	Literal(RegExp),
	Enum(Id),
	Intersection(Id),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
	Native(Native),
	Reference,
	Literal,
	Struct,
	Enum,
	Intersection,
}

impl Description {
	pub fn ty(&self) -> Type {
		match self {
			Self::Reference(_) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Native(n) => Type::Native(*n),
			Self::Literal(_) => Type::Literal,
			Self::Enum(_) => Type::Enum,
			Self::Intersection(_) => Type::Intersection,
		}
	}

	pub fn sub_layouts<F, D: Descriptions<F>>(&self, context: &Context<F, D>, causes: &Causes<F>) -> Result<Vec<SubLayout<F>>, Error<F>> where F: Clone + Ord {
		let mut sub_layouts = Vec::new();

		if let Description::Struct(fields_id) = self {
			let fields = context
				.require_list(*fields_id, causes.preferred().cloned())?
				.iter(context);
			for item in fields {
				let (object, causes) = item?.clone().into_parts();
				let field_id = match object {
					vocab::Object::Literal(_) => Err(Caused::new(
						error::LayoutLiteralField(*fields_id).into(),
						causes.preferred().cloned(),
					)),
					vocab::Object::Iri(id) => Ok(Id::Iri(id)),
					vocab::Object::Blank(id) => Ok(Id::Blank(id)),
				}?;
				let field = context.require_layout_field(field_id, causes.into_preferred())?;
				let field_layout_id = field.require_layout(field.causes())?;

				sub_layouts.push(SubLayout {
					layout: field_layout_id.clone(),
					connection: LayoutConnection::Field(field_id),
				});
			}
		}

		Ok(sub_layouts)
	}

	pub fn dependencies<F: Clone + Ord>(
		&self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>> {
		match self {
			Description::Enum(variant_list_id) => {
				let layouts = nodes
					.require_list(*variant_list_id, causes.preferred().cloned())?
					.iter(nodes)
					.map(|item| {
						let (object, variant_causes) = item?.clone().into_parts();
						let variant_id = match object {
							vocab::Object::Literal(_) => Err(Caused::new(
								error::LayoutLiteralField(id).into(),
								causes.preferred().cloned(),
							)),
							vocab::Object::Iri(id) => Ok(Id::Iri(id)),
							vocab::Object::Blank(id) => Ok(Id::Blank(id)),
						}?;

						let variant = nodes.require_layout_variant(
							variant_id,
							variant_causes.preferred().cloned(),
						)?;

						let layout = variant.layout().clone().try_map_with_causes(|layout_id| {
							Ok(*nodes
								.require_layout(
									*layout_id.inner(),
									layout_id.causes().preferred().cloned(),
								)?
								.inner())
						})?;

						Ok(layout.into_value().map(crate::Item::Layout))
					})
					.try_filter_collect()?;
				Ok(layouts)
			}
			Description::Intersection(layout_list_id) => {
				let layouts = nodes
					.require_list(*layout_list_id, causes.preferred().cloned())?
					.iter(nodes)
					.map(|item| {
						let (object, causes) = item?.clone().into_parts();
						let layout_id = match object {
							vocab::Object::Literal(_) => Err(Caused::new(
								error::LayoutLiteralIntersection(*layout_list_id).into(),
								causes.preferred().cloned(),
							)),
							vocab::Object::Iri(id) => Ok(Id::Iri(id)),
							vocab::Object::Blank(id) => Ok(Id::Blank(id)),
						}?;

						Ok(crate::Item::Layout(
							**nodes.require_layout(layout_id, causes.into_preferred())?,
						))
					})
					.try_collect()?;
				Ok(layouts)
			}
			_ => Ok(Vec::new()),
		}
	}
}

impl<F: Clone + Ord> PseudoDescription<F> for Description {
	type Error = Error<F>;

	fn as_standard(&self) -> Option<&Description> {
		Some(self)
	}

	fn sub_layouts<D: Descriptions<F>>(&self, context: &Context<F, D>, causes: &Causes<F>) -> Result<Vec<SubLayout<F>>, Self::Error> {
		self.sub_layouts(context, causes)
	}

	fn dependencies(
		&self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>> {
		self.dependencies(id, nodes, causes)
	}

	fn reduce(self, _id: Id, _name: &MaybeSet<vocab::Name, F>, _ty: &MaybeSet<Id, F>) -> Result<Description, Self::Error> {
		Ok(self)
	}
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

	pub fn require_ty(&self, cause: Option<Location<F>>) -> Result<&WithCauses<Id, F>, Error<F>> {
		self.ty
			.value_or_else(|| Caused::new(error::LayoutMissingType(self.id).into(), cause))
	}

	pub fn name(&self) -> Option<&WithCauses<vocab::Name, F>> {
		self.name.with_causes()
	}

	pub fn set_name(
		&mut self,
		name: vocab::Name,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.name.try_set(name, cause, |expected, because, found| {
			error::LayoutMismatchName {
				id: self.id,
				expected: expected.clone(),
				found,
				because: because.cloned(),
			}
			.into()
		})
	}

	pub fn description(&self) -> Option<&WithCauses<D, F>> {
		self.desc.with_causes()
	}

	/// Declare the type for which this layout is defined.
	pub fn set_type(&mut self, ty_ref: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.ty.try_set(ty_ref, cause, |expected, because, found| {
			error::LayoutMismatchType {
				id: self.id,
				expected: *expected,
				found,
				because: because.cloned(),
			}
			.into()
		})
	}

	pub fn set_description(
		&mut self,
		desc: D,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
		D: PartialEq
	{
		self.desc.try_set(desc, cause, |_expected, because, _found| {
			error::LayoutMismatchDescription {
				id: self.id,
				because: because.cloned(),
			}
			.into()
		})
	}
}

impl<F: Clone + Ord, D: PseudoDescription<F>> Definition<F, D> {
	pub fn set_native(&mut self, native: Native, cause: Option<Location<F>>) -> Result<(), Error<F>> {
		self.set_description(Description::Native(native).into(), cause)
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

	pub fn set_intersection(
		&mut self,
		types_list: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>> {
		self.set_description(Description::Intersection(types_list).into(), cause)
	}

	pub fn sub_layouts<C: Descriptions<F>>(&self, context: &Context<F, C>) -> Result<Vec<SubLayout<F>>, D::Error> {
		match self.desc.with_causes() {
			Some(desc) => desc.sub_layouts(context, desc.causes()),
			None => Ok(Vec::new())
		}
	}

	pub fn dependencies(
		&self,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, D::Error> {
		let desc = self.desc.with_causes().ok_or_else(|| {
			Caused::new(
				error::LayoutMissingDescription(self.id).into(),
				causes.preferred().cloned(),
			)
		})?;

		desc.dependencies(self.id, nodes, desc.causes())
	}

	/// Build a default name for this layout.
	pub fn default_name<C: Descriptions<F>>(
		&self,
		context: &Context<F, C>,
		parent_layouts: &[WithCauses<ParentLayout, F>],
		cause: Option<Location<F>>,
	) -> Result<Option<Caused<vocab::Name, F>>, Error<F>> {
		if let Id::Iri(iri) = self.id {
			if let Some(name) = iri.iri(context.vocabulary()).unwrap().path().file_name() {
				if let Ok(name) = vocab::Name::new(name) {
					return Ok(Some(Caused::new(name, cause)));
				}
			}
		}

		if let Some(Description::Literal(regexp)) = self.desc.value().and_then(D::as_standard) {
			if let Some(singleton) = regexp.as_singleton() {
				if let Ok(singleton_name) = vocab::Name::new(singleton) {
					let mut name = vocab::Name::new("const").unwrap();
					name.push_name(&singleton_name);
					return Ok(Some(Caused::new(name, cause)));
				}
			}
		}

		if parent_layouts.len() == 1 {
			let parent = &parent_layouts[0];
			let layout = context
				.require_layout(parent.layout, cause.clone())?
				.inner();

			if let LayoutConnection::Field(field_id) = parent.connection {
				let field = context
					.require_layout_field(field_id, cause.clone())?
					.inner();

				if let Some(layout_name) = layout.name() {
					if let Some(field_name) = field.name() {
						let mut name = layout_name.inner().clone();
						name.push_name(field_name);

						return Ok(Some(Caused::new(name, cause)));
					}
				}
			}
		}

		Ok(None)
	}

	pub fn reduce(self) -> Result<Definition<F>, D::Error> {
		let desc = self.desc.try_map(|d| d.reduce(self.id, &self.name, &self.ty))?;
		Ok(Definition {
			id: self.id,
			name: self.name,
			ty: self.ty,
			desc
		})
	}
}

/// Field/layout usage.
///
/// For a given layout, this structure define a field used inside the layout,
/// and the layout of this field.
pub struct Using<F> {
	/// Layout field.
	pub field: Id,

	/// Field layout.
	pub field_layout: WithCauses<Id, F>,
}

impl<F: Ord + Clone> crate::Build<F> for Definition<F> {
	type Target = treeldr::layout::Definition<F>;
	type Error = Error<F>;

	fn build(
		mut self,
		vocab: &crate::Vocabulary,
		nodes: &super::context::AllocatedNodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Error<F>> {
		use field::Build as BuildField;
		use variant::Build as BuildVariant;

		let ty_id = self.ty.ok_or_else(|| {
			Caused::new(
				error::LayoutMissingType(self.id).into(),
				causes.preferred().cloned(),
			)
		})?;
		let ty = nodes
			.require_type(*ty_id, ty_id.causes().preferred().cloned())?
			.clone_with_causes(ty_id.into_causes());

		let def_desc = self.desc.ok_or_else(|| {
			Caused::new(
				error::LayoutMissingDescription(self.id).into(),
				causes.preferred().cloned(),
			)
		})?;

		fn require_name<F>(
			id: Id,
			name: MaybeSet<vocab::Name, F>,
			causes: &Causes<F>,
		) -> Result<WithCauses<vocab::Name, F>, Error<F>>
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

		let desc = def_desc
			.try_map_with_causes::<_, Error<F>, _>(|d, desc_causes| match d {
				Description::Native(n) => Ok(treeldr::layout::Description::Native(n, self.name)),
				Description::Reference(layout_id) => {
					let layout_ref = *nodes
						.require_layout(layout_id, desc_causes.preferred().cloned())?
						.inner();
					Ok(treeldr::layout::Description::Reference(
						layout_ref, self.name,
					))
				}
				Description::Struct(fields_id) => {
					let name = require_name(self.id, self.name, &causes)?;
					let fields = nodes
						.require_list(fields_id, desc_causes.preferred().cloned())?
						.iter(nodes)
						.map(|item| {
							let (object, causes) = item?.clone().into_parts();
							let field_id = match object {
								vocab::Object::Literal(_) => Err(Caused::new(
									error::LayoutLiteralField(fields_id).into(),
									causes.preferred().cloned(),
								)),
								vocab::Object::Iri(id) => Ok(Id::Iri(id)),
								vocab::Object::Blank(id) => Ok(Id::Blank(id)),
							}?;

							let field =
								nodes.require_layout_field(field_id, causes.into_preferred())?;
							let node = nodes.get(field_id).unwrap();
							let label = node.label().map(String::from);
							let doc = node.documentation().clone();
							field.build(label, doc, vocab, nodes)
						})
						.try_collect()?;

					let strct = treeldr::layout::Struct::new(name, fields);
					Ok(treeldr::layout::Description::Struct(strct))
				}
				Description::Enum(options_id) => {
					let name = require_name(self.id, self.name, &causes)?;

					let variants: Vec<_> = nodes
						.require_list(options_id, desc_causes.preferred().cloned())?
						.iter(nodes)
						.map(|item| {
							let (object, variant_causes) = item?.clone().into_parts();
							let variant_id = match object {
								vocab::Object::Literal(_) => Err(Caused::new(
									error::LayoutLiteralField(self.id).into(),
									causes.preferred().cloned(),
								)),
								vocab::Object::Iri(id) => Ok(Id::Iri(id)),
								vocab::Object::Blank(id) => Ok(Id::Blank(id)),
							}?;

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
					let name = require_name(self.id, self.name, &causes)?;
					let lit = treeldr::layout::Literal::new(regexp, name, self.id.is_blank());
					Ok(treeldr::layout::Description::Literal(lit))
				}
				Description::Intersection(layout_list_id) => {
					let layouts = nodes
						.require_list(layout_list_id, desc_causes.preferred().cloned())?
						.iter(nodes)
						.map(|item| {
							let (object, causes) = item?.clone().into_parts();
							let layout_id = match object {
								vocab::Object::Literal(_) => Err(Caused::new(
									error::LayoutLiteralIntersection(layout_list_id).into(),
									causes.preferred().cloned(),
								)),
								vocab::Object::Iri(id) => Ok(Id::Iri(id)),
								vocab::Object::Blank(id) => Ok(Id::Blank(id)),
							}?;

							let layout_ref =
								nodes.require_layout(layout_id, causes.into_preferred())?;
							Ok(dependencies.layouts[layout_ref.index()].as_ref().unwrap())
						});

					let mut desc: Option<treeldr::layout::Description<F>> = None;
					for layout in layouts {
						let layout = layout?;

						desc = Some(match desc {
							Some(desc) => desc.intersected_with(
								self.id,
								layout.description_with_causes(),
								self.name.take(),
								dependencies.layouts,
							)?,
							None => layout.description().clone(),
						})
					}

					Ok(desc.unwrap())
				}
			})
			.map_err(Caused::flatten)?;

		Ok(treeldr::layout::Definition::new(self.id, ty, desc, causes))
	}
}

pub trait ComputeIntersection<F>: Sized {
	/// Intersects this type description with `other`.
	///
	/// If provided, `name` will override the name of the intersected type,
	/// otherwise the name of `self` is used.
	fn intersected_with(
		self,
		id: Id,
		other: &WithCauses<Self, F>,
		name: MaybeSet<Name, F>,
		built_layouts: &[Option<treeldr::layout::Definition<F>>],
	) -> Result<Self, Error<F>>;
}

impl<F: Clone + Ord> ComputeIntersection<F> for treeldr::layout::Description<F> {
	fn intersected_with(
		self,
		id: Id,
		other: &WithCauses<Self, F>,
		name: MaybeSet<Name, F>,
		built_layouts: &[Option<treeldr::layout::Definition<F>>],
	) -> Result<Self, Error<F>> {
		match (self, other.inner()) {
			(Self::Native(a, a_name), Self::Native(b, _)) if &a == b => {
				Ok(Self::Native(a, name.or(a_name)))
			}
			(Self::Reference(a, a_name), Self::Reference(b, _)) if &a == b => {
				Ok(Self::Reference(a, name.or(a_name)))
			}
			(Self::Literal(a), Self::Literal(b)) => {
				use literal::IntersectedWith;
				Ok(Self::Literal(a.intersected_with(
					id,
					b,
					name,
					other.causes().preferred(),
				)?))
			}
			(Self::Struct(a), Self::Struct(b)) => {
				use structure::IntersectedWith;
				Ok(Self::Struct(a.intersected_with(
					id,
					b,
					name,
					other.causes().preferred(),
				)?))
			}
			(Self::Enum(a), Self::Enum(b)) => {
				use enumeration::IntersectedWith;
				let e = a.intersected_with(id, b, name, other.causes().preferred())?;

				if e.variants().len() == 1 && e.variants()[0].layout().is_some() {
					let layout_ref = e.variants()[0].layout().unwrap();
					let mut desc = built_layouts[layout_ref.index()]
						.as_ref()
						.unwrap()
						.description()
						.clone();
					desc.set_name(e.name().clone(), e.name_causes().preferred().cloned());
					Ok(desc)
				} else {
					Ok(Self::Enum(e))
				}
			}
			_ => Err(Caused::new(
				error::LayoutIntersectionFailed { id }.into(),
				other.causes().preferred().cloned(),
			)),
		}
	}
}
