use crate::{Caused, MaybeSet, Id, WithCauses, Vocabulary, Feature, vocab, utils::TryCollect};
use super::Error;
use locspan::Location;

pub mod field;

pub use crate::layout::Type;
pub use crate::layout::Native;

/// Layout definition.
pub struct Definition<F> {
	name: MaybeSet<String, F>,
	ty: MaybeSet<Id, F>,
	desc: MaybeSet<Description, F>
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
	pub fn new() -> Self {
		Self {
			name: MaybeSet::default(),
			ty: MaybeSet::default(),
			desc: MaybeSet::default()
		}
	}

	/// Type for which the layout is defined.
	pub fn ty(&self) -> Option<&WithCauses<Id, F>> {
		self.ty.with_causes()
	}

	pub fn name(&self) -> Option<&WithCauses<String, F>> {
		self.name.with_causes()
	}

	pub fn set_name(&mut self, name: String, cause: Option<Location<F>>) -> Result<(), Caused<Error<F>, F>> where F: Ord {
		self.name.try_set(name, cause, |expected, because, found| todo!())
	}

	pub fn description(&self) -> Option<&WithCauses<Description, F>> {
		self.desc.with_causes()
	}

	/// Declare the type for which this layout is defined.
	pub fn set_type(
		&mut self,
		ty_ref: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Error<F>, F>> where F: Clone + Ord {
		self.ty.try_set(ty_ref, cause, |expected, because, found| todo!())
	}

	pub fn set_native(
		&mut self,
		native: Native,
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Mismatch<F>, F>> where F: Clone + Ord {
		self.desc.try_set(Description::Native(native), cause, |expected, because, found| todo!())
	}

	pub fn set_fields(
		&mut self,
		fields: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Mismatch<F>, F>> where F: Clone + Ord {
		self.desc.try_set(Description::Struct(fields), cause, |expected, because, found| todo!())
	}

	pub fn set_deref_to(
		&mut self,
		target: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Caused<Mismatch<F>, F>> where F: Clone + Ord {
		self.desc.try_set(Description::Reference(target), cause, |expected, because, found| todo!())
	}
}

impl<F: Ord + Clone> WithCauses<Definition<F>, F> {
	pub fn build(self, id: Id, vocab: &Vocabulary, nodes: &super::context::AllocatedNodes<F>) -> Result<crate::layout::Definition<F>, Caused<Error<F>, F>> {
		let (def, causes) = self.into_parts();

		let name = def.name.unwrap_or_else_try(|| {
			match id {
				Id::Iri(name) => {
					let iri = name.iri(vocab).unwrap();
					Ok(iri.path().file_name().ok_or_else(||
						Caused::new(Error::Unimplemented(Feature::Error("layout name cannot be inferred")), causes.preferred().cloned())
					)?.into())
				}
				Id::Blank(blank) => {
					Err(Caused::new(Error::Unimplemented(Feature::Error("layout name cannot be inferred from blank node label")), causes.preferred().cloned()))
				}
			}
		})?;
		
		let ty_id = def.ty.ok_or_else(|| Caused::new(Error::Unimplemented(Feature::Error("missing layout type")), causes.preferred().cloned()))?;
		let ty = nodes.require_type(*ty_id, ty_id.causes().preferred().cloned())?.clone_with_causes(ty_id.into_causes());

		let def_desc = def.desc.ok_or_else(|| Caused::new(Error::Unimplemented(Feature::Error("missing layout description")), causes.preferred().cloned()))?;
		let desc = def_desc.try_map_with_causes::<_, Caused<Error<F>, F>, _>(|d, desc_causes| match d {
			Description::Native(n) => Ok(crate::layout::Description::Native(n)),
			Description::Reference(layout_id) => {
				let layout_ref = *nodes.require_layout(id, desc_causes.preferred().cloned())?.inner();
				Ok(crate::layout::Description::Reference(layout_ref))
			}
			Description::Struct(id) => {
				let fields = nodes.require_list(id, desc_causes.preferred().cloned())?.iter(nodes).map(|item| {
					let (object, causes) = item?.into_parts();
					let field_id = match object {
						vocab::Object::Literal(_) => Err(Caused::new(Error::Unimplemented(Feature::Error("field is a literal value")), causes.preferred().cloned())),
						vocab::Object::Iri(id) => Ok(Id::Iri(id)),
						vocab::Object::Blank(id) => Ok(Id::Blank(id))
					}?;

					let field = nodes.require_layout_field(field_id, causes.into_preferred())?;
					field.build(nodes)
				}).try_collect()?;
				Ok(crate::layout::Description::Struct(fields))
			}
		})?;

		let mut result = crate::layout::Definition::new(
			id,
			name,
			ty,
			desc,
			causes
		);

		Ok(result)
	}
}

/// Layout mismatch error.
#[derive(Debug)]
pub enum Mismatch<F> {
	Type {
		expected: Type,
		found: Type,
		because: Option<Location<F>>,
	},
	FieldProperty {
		expected: Id,
		found: Id,
		because: Option<Location<F>>,
	},
	FieldName {
		expected: String,
		found: String,
		because: Option<Location<F>>,
	},
	FieldLayout {
		expected: Id,
		found: Id,
		because: Option<Location<F>>,
	},
	AttributeRequired {
		/// Is the field required?
		///
		/// If `true` then it is, and some other declaration is missing the `required` attribute.
		/// If `false` then it is not, and some other declaration is adding the attribute.
		required: bool,
		because: Option<Location<F>>,
	},
	AttributeFunctional {
		functional: bool,
		because: Option<Location<F>>,
	},
	MissingField {
		name: String,
		because: Option<Location<F>>,
	},
	AdditionalField {
		name: String,
		because: Option<Location<F>>,
	},
}