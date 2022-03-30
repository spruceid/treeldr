use crate::{error, utils::TryCollect, vocab, Caused, Causes, Error, Id, MaybeSet, Vocabulary, WithCauses};
use std::collections::HashSet;
use locspan::Location;

pub mod field;

pub use crate::layout::{literal::RegExp, Native, Type};

/// Layout definition.
pub struct Definition<F> {
	/// Identifier of the layout.
	id: Id,

	/// Optional name.
	/// 
	/// If not provided, the name is generated using the `generate_default_name`
	/// method. If it conflicts with another name or failed to be generated,
	/// then a name must be explicitly defined by the user.
	name: MaybeSet<String, F>,

	/// Type for which this layout is defined.
	ty: MaybeSet<Id, F>,

	/// Layout description.
	desc: MaybeSet<Description, F>,

	/// The fields having this layout as layout.
	/// 
	/// This is used to generate a default name for the layout if necessary.
	/// 
	/// ## Example
	/// 
	/// ```treeldr
	/// layout Struct {
	///   foo: Layout
	/// }
	/// ```
	/// 
	/// Here, `Layout` is only used in `foo` which is a field of `Struct`.
	/// A possible default name for `Layout` is hence `StructFoo`.
	uses: HashSet<Use>
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Use {
	layout: Id,
	field: Id
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Description {
	Native(Native),
	Struct(Id),
	Reference(Id),
	Literal(RegExp),
	Sum(Id)
}

impl Description {
	pub fn ty(&self) -> Type {
		match self {
			Self::Reference(_) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Native(n) => Type::Native(*n),
			Self::Literal(_) => Type::Literal,
			Self::Sum(_) => Type::Sum
		}
	}
}

impl<F> Definition<F> {
	pub fn new(id: Id) -> Self {
		Self {
			id,
			name: MaybeSet::default(),
			ty: MaybeSet::default(),
			desc: MaybeSet::default(),
			uses: HashSet::new()
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

	pub fn add_use(&mut self, layout: Id, field: Id) {
		self.uses.insert(Use { layout, field });
	}

	/// Build a default name for this layout.
	pub fn compute_default_name(
		&self,
		context: &super::Context<F>,
		cause: Option<Location<F>>
	) -> Result<Option<Caused<String, F>>, Error<F>> where F: Clone {
		if let Id::Iri(iri) = self.id {
			if let Some(name) = iri.iri(context.vocabulary()).unwrap().path().file_name() {
				return Ok(Some(Caused::new(name.to_string(), cause)))
			}
		}

		let ty = self.require_ty(cause.clone())?;
		if let Id::Iri(iri) = ty.inner() {
			if let Some(name) = iri.iri(context.vocabulary()).unwrap().path().file_name() {
				return Ok(Some(Caused::new(name.to_string(), cause)))
			}
		}

		if self.uses.len() == 1 {
			let u = self.uses.iter().next().unwrap();
			let layout = context.require_layout(u.layout, cause.clone())?.inner();
			let field = context.require_layout_field(u.field, cause.clone())?.inner();

			if let Some(layout_name) = layout.name() {
				if let Some(field_name) = field.name() {
					return Ok(Some(Caused::new(
						format!("{}{}", layout_name.inner(), field_name.inner()),
						cause
					)))
				}
			}
		}

		Ok(None)
	}

	pub fn name(&self) -> Option<&WithCauses<String, F>> {
		self.name.with_causes()
	}

	pub fn set_name(&mut self, name: String, cause: Option<Location<F>>) -> Result<(), Error<F>>
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


	pub fn description(&self) -> Option<&WithCauses<Description, F>> {
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
		desc: Description,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.desc.try_set(desc, cause, |expected, because, found| {
			error::LayoutMismatchDescription {
				id: self.id,
				expected: expected.clone(),
				found,
				because: because.cloned(),
			}
			.into()
		})
	}

	pub fn set_native(&mut self, native: Native, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.set_description(Description::Native(native), cause)
	}

	pub fn set_fields(&mut self, fields: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.set_description(Description::Struct(fields), cause)
	}

	pub fn set_deref_to(&mut self, target: Id, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.set_description(Description::Reference(target), cause)
	}

	pub fn set_literal(
		&mut self,
		regexp: RegExp,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		self.set_description(Description::Literal(regexp), cause)
	}
}

impl<F: Ord + Clone> WithCauses<Definition<F>, F> {
	pub fn compute_uses(&self, nodes: &super::Context<F>) -> Result<Vec<(Id, WithCauses<Id, F>)>, Error<F>> {
		let mut uses = Vec::new();
		
		if let Some(desc) = self.desc.with_causes() {
			if let Description::Struct(fields_id) = desc.inner() {
				let fields = nodes.require_list(*fields_id, desc.causes().preferred().cloned())?.iter(nodes);
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
					let field = nodes.require_layout_field(field_id, causes.into_preferred())?;
					let field_layout_id = field.require_layout(field.causes())?;

					// let field_layout = nodes.require_layout_mut(*field_layout_id.inner(), field_layout_id.causes().preferred().cloned())?;
					// field_layout.add_use(self.id, field_id);
					uses.push((field_id, field_layout_id.clone()));
				}
			}
		}

		Ok(uses)
	}

	pub fn build(
		self,
		id: Id,
		vocab: &Vocabulary,
		nodes: &super::context::AllocatedNodes<F>,
	) -> Result<crate::layout::Definition<F>, Error<F>> {
		let (def, causes) = self.into_parts();

		let ty_id = def.ty.ok_or_else(|| {
			Caused::new(
				error::LayoutMissingType(id).into(),
				causes.preferred().cloned(),
			)
		})?;
		let ty = nodes
			.require_type(*ty_id, ty_id.causes().preferred().cloned())?
			.clone_with_causes(ty_id.into_causes());

		let def_desc = def.desc.ok_or_else(|| {
			Caused::new(
				error::LayoutMissingDescription(id).into(),
				causes.preferred().cloned(),
			)
		})?;

		fn require_name<F>(
			id: Id,
			name: MaybeSet<String, F>,
			causes: &Causes<F>
		) -> Result<WithCauses<String, F>, Error<F>> where F: Clone {
			name.ok_or_else(|| {
				Caused::new(
					error::LayoutMissingName(id).into(),
					causes.preferred().cloned(),
				)
			})
		}

		let desc = def_desc
			.try_map_with_causes::<_, Error<F>, _>(|d, desc_causes| match d {
				Description::Native(n) => Ok(crate::layout::Description::Native(n, def.name)),
				Description::Reference(layout_id) => {
					let layout_ref = *nodes
						.require_layout(layout_id, desc_causes.preferred().cloned())?
						.inner();
					Ok(crate::layout::Description::Reference(layout_ref, def.name))
				}
				Description::Struct(id) => {
					let name = require_name(def.id, def.name, &causes)?;
					let fields = nodes
						.require_list(id, desc_causes.preferred().cloned())?
						.iter(nodes)
						.map(|item| {
							let (object, causes) = item?.clone().into_parts();
							let field_id = match object {
								vocab::Object::Literal(_) => Err(Caused::new(
									error::LayoutLiteralField(id).into(),
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

					let strct = crate::layout::Struct::new(name, fields);

					Ok(crate::layout::Description::Struct(strct))
				}
				Description::Sum(options) => {
					let name = require_name(def.id, def.name, &causes)?;

					todo!()
				}
				Description::Literal(regexp) => {
					let name = require_name(def.id, def.name, &causes)?;
					let lit = crate::layout::Literal::new(regexp, name, def.id.is_blank());
					Ok(crate::layout::Description::Literal(lit))
				}
			})
			.map_err(Caused::flatten)?;

		Ok(crate::layout::Definition::new(id, ty, desc, causes))
	}
}
