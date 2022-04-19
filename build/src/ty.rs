use crate::{error, Error};
use derivative::Derivative;
use locspan::Location;
use std::collections::HashMap;
use treeldr::{vocab, Caused, Causes, Id, WithCauses};

pub use treeldr::ty::Kind;

/// Type definition.
pub enum Definition<F> {
	/// Normal type.
	Normal(WithCauses<Normal<F>, F>),

	/// Union/sum type.
	Union(WithCauses<Id, F>),

	/// Intersection type.
	Intersection(WithCauses<Id, F>),
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
			Self::Intersection(_) => Kind::Intersection,
		}
	}

	pub fn causes(&self) -> &Causes<F> {
		match self {
			Self::Normal(n) => n.causes(),
			Self::Union(u) => u.causes(),
			Self::Intersection(i) => i.causes(),
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
			_ => Err(Error::new(
				error::TypeMismatchKind {
					id,
					expected: self.kind(),
					found: Kind::Normal,
					because: self.causes().preferred().cloned(),
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
			Self::Intersection(i) => Err(Error::new(
				error::TypeMismatchKind {
					id,
					expected: Kind::Intersection,
					found: Kind::Union,
					because: i.causes().preferred().cloned(),
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn declare_intersection(
		&mut self,
		id: Id,
		types_ref: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		match self {
			Self::Intersection(i) => {
				if *i.inner() == types_ref {
					i.add_opt_cause(cause);
					Ok(())
				} else {
					Err(Error::new(
						error::TypeMismatchIntersection {
							id,
							expected: *i.inner(),
							found: types_ref,
							because: i.causes().preferred().cloned(),
						}
						.into(),
						cause,
					))
				}
			}
			Self::Normal(n) => {
				if n.is_empty() {
					*self = Self::Intersection(WithCauses::new(types_ref, cause));
					Ok(())
				} else {
					Err(Error::new(
						error::TypeMismatchKind {
							id,
							expected: Kind::Normal,
							found: Kind::Intersection,
							because: n.causes().preferred().cloned(),
						}
						.into(),
						cause,
					))
				}
			}
			Self::Union(u) => Err(Error::new(
				error::TypeMismatchKind {
					id,
					expected: Kind::Union,
					found: Kind::Intersection,
					because: u.causes().preferred().cloned(),
				}
				.into(),
				cause,
			)),
		}
	}
}

impl<F: Ord + Clone> crate::Build<F> for Definition<F> {
	type Target = treeldr::ty::Definition<F>;
	type Error = Error<F>;

	fn build(
		self,
		id: Id,
		_vocab: &crate::Vocabulary,
		nodes: &super::context::AllocatedNodes<F>,
		_dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Error<F>> {
		let desc = match self {
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

				treeldr::ty::Description::Union(treeldr::ty::Union::new(options))
			}
			Definition::Intersection(types_id) => {
				use std::collections::hash_map::Entry;
				let (types_id, types_causes) = types_id.into_parts();
				let mut types = HashMap::new();

				let items = nodes
					.require_list(types_id, types_causes.preferred().cloned())?
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

					let (ty, ty_causes) = nodes
						.require_type(option_id, causes.into_preferred())?
						.clone()
						.into_parts();

					match types.entry(ty) {
						Entry::Vacant(entry) => {
							entry.insert(ty_causes);
						}
						Entry::Occupied(mut entry) => {
							entry.get_mut().extend(ty_causes);
						}
					}
				}

				treeldr::ty::Description::Intersection(treeldr::ty::Intersection::new(types))
			}
		};

		Ok(treeldr::ty::Definition::new(id, desc, causes))
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
	) -> Result<treeldr::ty::Description<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let mut result = treeldr::ty::Normal::new();

		for (prop_id, prop_causes) in self.properties {
			let prop_ref = nodes.require_property(prop_id, prop_causes.preferred().cloned())?;
			result.insert_property(*prop_ref.inner(), prop_causes)
		}

		Ok(treeldr::ty::Description::Normal(result))
	}
}
