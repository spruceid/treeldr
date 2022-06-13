use crate::{error, utils::TryCollect, Error, ObjectToId};
use locspan::Location;
use std::collections::BTreeMap;
use treeldr::{Causes, Id, MaybeSet};

pub mod data;
mod normal;
mod restriction;

pub use data::DataType;
pub use normal::*;
pub use restriction::*;
pub use treeldr::ty::Kind;

/// Type definition.
#[derive(Clone)]
pub enum Description<F> {
	Data(DataType<F>),

	/// Normal type.
	Normal(Normal<F>),

	/// Union/sum type.
	Union(Id),

	/// Intersection type.
	Intersection(Id),

	/// Property restriction.
	Restriction(Restriction<F>),

	/// Enumeration.
	Enumeration(Id)
}

impl<F: Clone + Ord> Description<F> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Data(_) => Kind::Data,
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
			Self::Intersection(_) => Kind::Intersection,
			Self::Restriction(_) => Kind::Restriction,
			Self::Enumeration(_) => Kind::Enumeration
		}
	}

	fn dependencies(
		&self,
		_id: Id,
		nodes: &super::context::allocated::Nodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<crate::Item<F>>, Error<F>> {
		let list_id = match self {
			Description::Union(list_id) => Some(*list_id),
			Description::Intersection(list_id) => Some(*list_id),
			Description::Data(dt) => return dt.dependencies(nodes),
			_ => None,
		};

		match list_id {
			Some(list_id) => {
				let dependencies = nodes
					.require_list(list_id, causes.preferred().cloned())?
					.iter(nodes)
					.map(|item| {
						let (object, ty_causes) = item?.clone().into_parts();
						let ty_id = object.into_id(causes.preferred())?;

						let ty_ref = *nodes
							.require_type(ty_id, ty_causes.preferred().cloned())?
							.inner();

						Ok(crate::Item::Type(ty_ref))
					})
					.try_collect()?;
				Ok(dependencies)
			}
			None => Ok(Vec::new()),
		}
	}

	fn build(
		self,
		id: Id,
		nodes: &super::context::allocated::Nodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<treeldr::ty::Description<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let desc = match self {
			Self::Data(d) => d.build(id, nodes, dependencies)?,
			Self::Normal(n) => n.build(nodes)?,
			Self::Union(options_id) => {
				use std::collections::btree_map::Entry;
				let mut options = BTreeMap::new();

				let items = nodes
					.require_list(options_id, causes.preferred().cloned())?
					.iter(nodes);
				for item in items {
					let (object, causes) = item?.clone().into_parts();
					let option_id = object.into_id(causes.preferred())?;

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

				treeldr::ty::Description::Union(treeldr::ty::Union::new(options, |ty_ref| {
					dependencies.ty(ty_ref)
				}))
			}
			Description::Intersection(types_id) => {
				use std::collections::btree_map::Entry;
				let mut types = BTreeMap::new();

				let items = nodes
					.require_list(types_id, causes.preferred().cloned())?
					.iter(nodes);
				for item in items {
					let (object, causes) = item?.clone().into_parts();
					let option_id = object.into_id(causes.preferred())?;

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

				match treeldr::ty::Intersection::new(types, |ty_ref| dependencies.ty(ty_ref)) {
					Ok(intersection) => treeldr::ty::Description::Intersection(intersection),
					Err(_) => treeldr::ty::Description::Empty,
				}
			}
			Description::Restriction(r) => r.build(nodes)?,
			Description::Enumeration(values_id) => {
				use std::collections::btree_map::Entry;
				use crate::ObjectToValue;
				let mut values = BTreeMap::new();

				let items = nodes
					.require_list(values_id, causes.preferred().cloned())?
					.iter(nodes);
				for item in items {
					let (object, causes) = item?.clone().into_parts();
					let value = object.into_value(causes.preferred())?;

					match values.entry(value) {
						Entry::Vacant(entry) => {
							entry.insert(causes);
						}
						Entry::Occupied(mut entry) => {
							entry.get_mut().extend(causes);
						}
					}
				}

				treeldr::ty::Description::Enumeration(
					treeldr::ty::Enumeration::new(values)
				)
			}
		};

		Ok(desc)
	}
}

pub trait PseudoDescription<F>: Clone + From<Description<F>> {
	fn as_standard(&self) -> Option<&Description<F>>;

	fn as_standard_mut(&mut self) -> Option<&mut Description<F>>;
}

impl<F: Clone> PseudoDescription<F> for Description<F> {
	fn as_standard(&self) -> Option<&Description<F>> {
		Some(self)
	}

	fn as_standard_mut(&mut self) -> Option<&mut Description<F>> {
		Some(self)
	}
}

#[derive(Clone)]
pub struct Definition<F, D = Description<F>> {
	/// Identifier of the type.
	id: Id,

	/// Type description.
	desc: MaybeSet<D, F>,
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

	pub fn try_map<U, E>(self, f: impl FnOnce(D) -> Result<U, E>) -> Result<Definition<F, U>, E> {
		Ok(Definition {
			id: self.id,
			desc: self.desc.try_map(f)?,
		})
	}
}

impl<F, D: PseudoDescription<F>> Definition<F, D> {
	pub fn require_datatype(&self, cause: Option<Location<F>>) -> Result<&DataType<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let because = self.desc.causes().unwrap().preferred().cloned();
		match self.desc.value().and_then(|d| d.as_standard()) {
			Some(Description::Data(d)) => Ok(d),
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

	pub fn require_datatype_mut(
		&mut self,
		cause: Option<Location<F>>,
	) -> Result<&mut DataType<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		self.desc.set_once(cause.clone(), || {
			Description::Data(DataType::default()).into()
		});
		let because = self.desc.causes().unwrap().preferred().cloned();
		match self.desc.value_mut().unwrap().as_standard_mut() {
			Some(Description::Data(d)) => Ok(d),
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
					found: Some(Kind::Data),
					because,
				}
				.into(),
				cause,
			)),
			None => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: None,
					found: Some(Kind::Data),
					because,
				}
				.into(),
				cause,
			)),
		}
	}

	/// Declare that this type is a datatype.
	pub fn declare_datatype(&mut self, cause: Option<Location<F>>) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.desc.set_once(cause.clone(), || {
			Description::Data(DataType::default()).into()
		});
		let because = self.desc.causes().unwrap().preferred().cloned();
		match self.desc.value_mut().unwrap().as_standard() {
			Some(Description::Data(_)) => Ok(()),
			Some(other) => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: Some(other.kind()),
					found: Some(Kind::Data),
					because,
				}
				.into(),
				cause,
			)),
			None => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: None,
					found: Some(Kind::Data),
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
			Some(Description::Union(r)) => {
				if *r == options_ref {
					Ok(())
				} else {
					todo!()
				}
			}
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
			Some(Description::Intersection(r)) => {
				if *r == types_ref {
					Ok(())
				} else {
					todo!()
				}
			}
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

	pub fn declare_restriction(
		&mut self,
		restriction: Restriction<F>,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		let mut restriction = Some(restriction);
		self.desc.set_once(cause.clone(), || {
			Description::Restriction(restriction.take().unwrap()).into()
		});
		match restriction {
			Some(restriction) => {
				let because = self.desc.causes().unwrap().preferred().cloned();
				match self.desc.value_mut().unwrap().as_standard() {
					Some(Description::Restriction(r)) => {
						if *r == restriction {
							Ok(())
						} else {
							todo!()
						}
					}
					Some(other) => Err(Error::new(
						error::TypeMismatchKind {
							id: self.id,
							expected: Some(other.kind()),
							found: Some(Kind::Restriction),
							because,
						}
						.into(),
						cause,
					)),
					None => Err(Error::new(
						error::TypeMismatchKind {
							id: self.id,
							expected: None,
							found: Some(Kind::Restriction),
							because,
						}
						.into(),
						cause,
					)),
				}
			}
			None => Ok(()),
		}
	}

	pub fn declare_enumeration(
		&mut self,
		values_ref: Id,
		cause: Option<Location<F>>,
	) -> Result<(), Error<F>>
	where
		F: Ord + Clone,
	{
		self.desc
			.set_once(cause.clone(), || Description::Enumeration(values_ref).into());
		let because = self.desc.causes().unwrap().preferred().cloned();
		match self.desc.value_mut().unwrap().as_standard() {
			Some(Description::Union(r)) => {
				if *r == values_ref {
					Ok(())
				} else {
					todo!()
				}
			}
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
}

impl<F: Clone + Ord> Definition<F> {
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
}

impl<F: Clone + Ord> crate::Build<F> for Definition<F> {
	type Target = treeldr::ty::Definition<F>;

	fn build(
		self,
		nodes: &mut super::context::allocated::Nodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Error<F>> {
		let desc = match self.desc.unwrap() {
			Some(desc) => {
				let (desc, desc_causes) = desc.into_parts();
				desc.build(self.id, nodes, dependencies, desc_causes)?
			}
			None => treeldr::ty::Description::Normal(treeldr::ty::Normal::new()),
		};

		Ok(treeldr::ty::Definition::new(self.id, desc, causes))
	}
}
