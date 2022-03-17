use crate::{error, utils::TryCollect, vocab, Caused, Error, Id, MaybeSet, Vocabulary, WithCauses};
use locspan::Location;

pub mod field;

pub use crate::layout::Native;
pub use crate::layout::Type;

/// Layout definition.
pub struct Definition<F> {
	id: Id,
	name: MaybeSet<String, F>,
	ty: MaybeSet<Id, F>,
	desc: MaybeSet<Description, F>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Description {
	Native(Native),
	Struct(Id),
	Reference(Id),
}

impl Description {
	pub fn ty(&self) -> Type {
		match self {
			Self::Reference(_) => Type::Reference,
			Self::Struct(_) => Type::Struct,
			Self::Native(n) => Type::Native(*n),
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
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<&WithCauses<Id, F>> {
		self.ty.with_causes()
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
				expected: *expected,
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
}

impl<F: Ord + Clone> WithCauses<Definition<F>, F> {
	pub fn build(
		self,
		id: Id,
		vocab: &Vocabulary,
		nodes: &super::context::AllocatedNodes<F>,
	) -> Result<crate::layout::Definition<F>, Error<F>> {
		let (def, causes) = self.into_parts();

		let name = def.name.unwrap_or_else_try(|| match id {
			Id::Iri(name) => {
				let iri = name.iri(vocab).unwrap();
				Ok(iri
					.path()
					.file_name()
					.ok_or_else(|| {
						Caused::new(
							error::LayoutMissingName(id).into(),
							causes.preferred().cloned(),
						)
					})?
					.into())
			}
			Id::Blank(_) => Err(Caused::new(
				error::LayoutMissingName(id).into(),
				causes.preferred().cloned(),
			)),
		})?;

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
		let desc = def_desc
			.try_map_with_causes::<_, Error<F>, _>(|d, desc_causes| match d {
				Description::Native(n) => Ok(crate::layout::Description::Native(n)),
				Description::Reference(layout_id) => {
					let layout_ref = *nodes
						.require_layout(layout_id, desc_causes.preferred().cloned())?
						.inner();
					Ok(crate::layout::Description::Reference(layout_ref))
				}
				Description::Struct(id) => {
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
							let doc = nodes.get(field_id).unwrap().documentation().clone();
							field.build(doc, vocab, nodes)
						})
						.try_collect()?;
					Ok(crate::layout::Description::Struct(fields))
				}
			})
			.map_err(Caused::flatten)?;

		Ok(crate::layout::Definition::new(id, name, ty, desc, causes))
	}
}
