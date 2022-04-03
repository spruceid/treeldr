use super::Error;
use crate::{error, vocab, Caused, Causes, Id, WithCauses};
use derivative::Derivative;
use locspan::Location;
use std::collections::HashMap;

pub use crate::ty::Kind;

/// Type definition.
pub enum Definition<F> {
	/// Normal type.
	Normal(WithCauses<Normal<F>, F>),

	/// Union/sum type.
	Union(WithCauses<Id, F>),
}

impl<F> Default for Definition<F> {
	fn default() -> Self {
		Self::Normal(WithCauses::without_causes(Normal::default()))
	}
}

impl<F> Definition<F> {
	/// Create a new type.
	///
	/// By default, a normal type is created.
	/// It can later be changed into a non-normal type as long as no properties
	/// have been defined on it.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn kind(&self) -> Kind {
		match self {
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
		}
	}

	/// Declare a property of type.
	///
	/// The type must be normal.
	pub fn declare_property(
		&mut self,
		id: Id,
		prop_ref: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		match self {
			Self::Normal(n) => {
				n.add_opt_cause(cause.clone());
				n.declare_property(prop_ref, cause);
				Ok(())
			}
			Self::Union(u) => Err(Error::new(
				error::TypeMismatchKind {
					id,
					expected: Kind::Union,
					found: Kind::Normal,
					because: u.causes().preferred().cloned(),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn declare_union(
		&mut self,
		id: Id,
		options_ref: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		match self {
			Self::Union(u) => {
				if *u.inner() == options_ref {
					u.add_opt_cause(cause);
					Ok(())
				} else {
					Err(Error::new(
						error::TypeMismatchUnion {
							id,
							expected: *u.inner(),
							found: options_ref,
							because: u.causes().preferred().cloned(),
						}
						.into(),
						cause,
					))
				}
			}
			Self::Normal(n) => {
				if n.is_empty() {
					*self = Self::Union(WithCauses::new(options_ref, cause));
					Ok(())
				} else {
					Err(Error::new(
						error::TypeMismatchKind {
							id,
							expected: Kind::Normal,
							found: Kind::Union,
							because: n.causes().preferred().cloned(),
						}
						.into(),
						cause,
					))
				}
			}
		}
	}
}

impl<F: Ord + Clone> WithCauses<Definition<F>, F> {
	pub fn build(
		self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
	) -> Result<crate::ty::Definition<F>, Error<F>> {
		let (def, causes) = self.into_parts();

		let desc = match def {
			Definition::Normal(n) => n.into_inner().build(nodes)?,
			Definition::Union(options_id) => {
				use std::collections::hash_map::Entry;
				let (options_id, options_causes) = options_id.into_parts();
				let mut options = HashMap::new();

				let items = nodes
					.require_list(options_id, options_causes.preferred().cloned())?
					.iter(nodes);
				for item in items {
					let (object, causes) = item?.clone().into_parts();
					let option_id = match object {
						vocab::Object::Literal(_) => Err(Caused::new(
							error::TypeUnionLiteralOption(id).into(),
							causes.preferred().cloned(),
						)),
						vocab::Object::Iri(id) => Ok(Id::Iri(id)),
						vocab::Object::Blank(id) => Ok(Id::Blank(id)),
					}?;

					let (option_ty, option_causes) = nodes
						.require_type(option_id, causes.into_preferred())?
						.clone()
						.into_parts();

					match options.entry(option_ty) {
						Entry::Vacant(entry) => {
							entry.insert(option_causes);
						}
						Entry::Occupied(mut entry) => {
							entry.get_mut().extend(option_causes);
						}
					}
				}

				crate::ty::Description::Union(crate::ty::Union::new(options))
			}
		};

		Ok(crate::ty::Definition::new(id, desc, causes))
	}
}

/// Normal type definition.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Normal<F> {
	/// Properties.
	properties: HashMap<Id, Causes<F>>,
}

impl<F> Normal<F> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_empty(&self) -> bool {
		self.properties.is_empty()
	}

	pub fn properties(&self) -> impl Iterator<Item = (Id, &Causes<F>)> {
		self.properties.iter().map(|(p, c)| (*p, c))
	}

	pub fn declare_property(&mut self, prop_ref: Id, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		use std::collections::hash_map::Entry;
		match self.properties.entry(prop_ref) {
			Entry::Vacant(entry) => {
				entry.insert(cause.into());
			}
			Entry::Occupied(mut entry) => {
				if let Some(cause) = cause {
					entry.get_mut().add(cause)
				}
			}
		}
	}

	pub fn build(
		self,
		nodes: &super::context::AllocatedNodes<F>,
	) -> Result<crate::ty::Description<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let mut result = crate::ty::Normal::new();

		for (prop_id, prop_causes) in self.properties {
			let prop_ref = nodes.require_property(prop_id, prop_causes.preferred().cloned())?;
			result.insert_property(*prop_ref.inner(), prop_causes)
		}

		Ok(crate::ty::Description::Normal(result))
	}
}
