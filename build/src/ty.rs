use crate::{error, Error, utils::TryCollect};
use derivative::Derivative;
use locspan::Location;
use std::collections::HashMap;
use treeldr::{vocab, Caused, Causes, Id, MaybeSet};

pub use treeldr::ty::Kind;

pub struct Definition<F, D = Description<F>> {
	/// Identifier of the type.
	id: Id,

	/// Type description.
	desc: MaybeSet<D, F>,
}

pub trait PseudoDescription<F>: From<Description<F>> {
	type Error: From<Error<F>>;

	fn as_standard(&self) -> Option<&Description<F>>;

	fn as_standard_mut(&mut self) -> Option<&mut Description<F>>;

	fn dependencies(
		&self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Self::Error>;

	fn build(
		self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<treeldr::ty::Description<F>, Self::Error>;
}

/// Type definition.
pub enum Description<F> {
	/// Normal type.
	Normal(Normal<F>),

	/// Union/sum type.
	Union(Id),

	/// Intersection type.
	Intersection(Id),
}

impl<F: Clone + Ord> Description<F> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
			Self::Intersection(_) => Kind::Intersection,
		}
	}

	fn dependencies(
		&self,
		_id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>> {
		let list_id = match self {
			Description::Union(list_id) => Some(*list_id),
			Description::Intersection(list_id) => Some(*list_id),
			_ => None
		};

		match list_id {
			Some(list_id) => {
				let dependencies = nodes
					.require_list(list_id, causes.preferred().cloned())?
					.iter(nodes)
					.map(|item| {
						let (object, ty_causes) = item?.clone().into_parts();
						let ty_id = match object {
							vocab::Object::Literal(lit) => Err(Caused::new(
								error::LiteralUnexpected(lit).into(),
								causes.preferred().cloned(),
							)),
							vocab::Object::Iri(id) => Ok(Id::Iri(id)),
							vocab::Object::Blank(id) => Ok(Id::Blank(id)),
						}?;
		
						let ty_ref = *nodes
							.require_type(
								ty_id,
								ty_causes.preferred().cloned(),
							)?
							.inner();
		
						Ok(crate::Item::Type(ty_ref))
					})
					.try_collect()?;
				Ok(dependencies)
			}
			None => Ok(Vec::new())
		}
	}

	fn build(
		self,
		_id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<treeldr::ty::Description<F>, Error<F>> where F: Clone + Ord {
		let desc = match self {
			Self::Normal(n) => n.build(nodes)?,
			Self::Union(options_id) => {
				use std::collections::hash_map::Entry;
				let mut options = HashMap::new();

				let items = nodes
					.require_list(options_id, causes.preferred().cloned())?
					.iter(nodes);
				for item in items {
					let (object, causes) = item?.clone().into_parts();
					let option_id = match object {
						vocab::Object::Literal(lit) => Err(Caused::new(
							error::LiteralUnexpected(lit).into(),
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

				treeldr::ty::Description::Union(treeldr::ty::Union::new(
					options,
					|ty_ref| dependencies.ty(ty_ref)
				))
			}
			Description::Intersection(types_id) => {
				use std::collections::hash_map::Entry;
				let mut types = HashMap::new();

				let items = nodes
					.require_list(types_id, causes.preferred().cloned())?
					.iter(nodes);
				for item in items {
					let (object, causes) = item?.clone().into_parts();
					let option_id = match object {
						vocab::Object::Literal(lit) => Err(Caused::new(
							error::LiteralUnexpected(lit).into(),
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

				match treeldr::ty::Intersection::new(
					types,
					|ty_ref| dependencies.ty(ty_ref)
				) {
					Ok(intersection) => treeldr::ty::Description::Intersection(intersection),
					Err(_) => treeldr::ty::Description::Empty
				}
			}
		};

		Ok(desc)
	}
}

impl<F: Clone + Ord> PseudoDescription<F> for Description<F> {
	type Error = Error<F>;

	fn as_standard(&self) -> Option<&Description<F>> {
		Some(self)
	}

	fn as_standard_mut(&mut self) -> Option<&mut Description<F>> {
		Some(self)
	}

	fn dependencies(
		&self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>> {
		self.dependencies(id, nodes, causes)
	}

	fn build(
		self,
		id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<treeldr::ty::Description<F>, Self::Error> {
		self.build(id, nodes, dependencies, causes)
	}
}

impl<F, D> Definition<F, D> {
	/// Create a new type.
	///
	/// By default, a normal type is created.
	/// It can later be changed into a non-normal type as long as no properties
	/// have been defined on it.
	pub fn new(id: Id) -> Self {
		Self {
			id,
			desc: MaybeSet::default(),
		}
	}

	pub fn description(&self) -> &MaybeSet<D, F> {
		&self.desc
	}

	// pub fn kind(&self) -> Option<Kind> {
	// 	self.desc.value().map(Description::kind)
	// }
}

impl<F, D: PseudoDescription<F>> Definition<F, D> {
	pub fn dependencies(
		&self,
		nodes: &super::context::AllocatedNodes<F>,
		_causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, D::Error>
	where
		F: Clone + Ord,
	{
		match self.desc.with_causes() {
			Some(desc) => desc.dependencies(self.id, nodes, desc.causes()),
			None => Ok(Vec::new()),
		}
	}

	pub fn require_normal_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<&mut Normal<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		self.desc
			.set_once(cause.clone(), || Description::Normal(Normal::new()).into());
		let because = self.desc.causes().unwrap().preferred().cloned();
		match self.desc.value_mut().unwrap().as_standard_mut() {
			Some(Description::Normal(n)) => Ok(n),
			Some(other) => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: Some(other.kind()),
					found: Some(Kind::Normal),
					because,
				}
				.into(),
				cause,
			)),
			None => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: None,
					found: Some(Kind::Normal),
					because,
				}
				.into(),
				cause,
			)),
		}
	}

	/// Declare a property of type.
	///
	/// The type must be normal.
	pub fn declare_property(
		&mut self,
		prop_ref: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		let n = self.require_normal_mut(cause.clone())?;
		n.declare_property(prop_ref, cause);
		Ok(())
	}

	pub fn declare_union(
		&mut self,
		options_ref: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.desc
			.set_once(cause.clone(), || Description::Union(options_ref).into());
		let because = self.desc.causes().unwrap().preferred().cloned();
		match self.desc.value_mut().unwrap().as_standard() {
			Some(Description::Union(_)) => Ok(()),
			Some(other) => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: Some(other.kind()),
					found: Some(Kind::Union),
					because,
				}
				.into(),
				cause,
			)),
			None => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: None,
					found: Some(Kind::Union),
					because,
				}
				.into(),
				cause,
			)),
		}
	}

	pub fn declare_intersection(
		&mut self,
		types_ref: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.desc.set_once(cause.clone(), || {
			Description::Intersection(types_ref).into()
		});
		let because = self.desc.causes().unwrap().preferred().cloned();
		match self.desc.value_mut().unwrap().as_standard() {
			Some(Description::Intersection(_)) => Ok(()),
			Some(other) => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: Some(other.kind()),
					found: Some(Kind::Intersection),
					because,
				}
				.into(),
				cause,
			)),
			None => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: None,
					found: Some(Kind::Intersection),
					because,
				}
				.into(),
				cause,
			)),
		}
	}
}

impl<F, D: PseudoDescription<F>> crate::Build<F> for Definition<F, D> {
	type Target = treeldr::ty::Definition<F>;
	type Error = D::Error;

	fn build(
		self,
		nodes: &super::context::AllocatedNodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Self::Error> {
		let desc = match self.desc.unwrap() {
			Some(desc) => {
				let (desc, desc_causes) = desc.into_parts();
				desc.build(
					self.id,
					nodes,
					dependencies,
					desc_causes
				)?
			}
			None => treeldr::ty::Description::Normal(treeldr::ty::Normal::new()),
		};

		Ok(treeldr::ty::Definition::new(self.id, desc, causes))
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
