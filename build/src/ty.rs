use crate::{error, utils::TryCollect, Error, ObjectToId};
use std::collections::BTreeMap;
use treeldr::{Id, MetaOption, metadata::Merge};
use locspan::Meta;

pub mod data;
mod normal;
mod restriction;

pub use data::DataType;
pub use normal::*;
pub use restriction::*;
pub use treeldr::ty::Kind;

/// Type definition.
#[derive(Clone)]
pub enum Description<M> {
	Data(DataType<M>),

	/// Normal type.
	Normal(Normal<M>),

	/// Union/sum type.
	Union(Id),

	/// Intersection type.
	Intersection(Id),

	/// Property restriction.
	Restriction(Restriction<M>),
}

impl<M: Clone> Description<M> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Data(_) => Kind::Data,
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
			Self::Intersection(_) => Kind::Intersection,
			Self::Restriction(_) => Kind::Restriction,
		}
	}

	fn dependencies(
		&self,
		_id: Id,
		nodes: &super::context::allocated::Nodes<M>,
		causes: &M,
	) -> Result<Vec<crate::Item<M>>, Error<M>> {
		let list_id = match self {
			Description::Union(list_id) => Some(*list_id),
			Description::Intersection(list_id) => Some(*list_id),
			Description::Data(dt) => return dt.dependencies(nodes),
			_ => None,
		};

		match list_id {
			Some(list_id) => {
				let dependencies = nodes
					.require_list(list_id, causes)?
					.iter(nodes)
					.map(|item| {
						let Meta(object, ty_causes) = item?.clone();
						let ty_id = object.into_id(causes)?;

						let ty_ref = **nodes
							.require_type(ty_id, &ty_causes)?;

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
		nodes: &super::context::allocated::Nodes<M>,
		dependencies: crate::Dependencies<M>,
		causes: M,
	) -> Result<treeldr::ty::Description<M>, Error<M>>
	where
		M: Clone + Merge,
	{
		let desc = match self {
			Self::Data(d) => d.build(id, nodes, dependencies)?,
			Self::Normal(n) => n.build(nodes)?,
			Self::Union(options_id) => {
				use std::collections::btree_map::Entry;
				let mut options = BTreeMap::new();

				let items = nodes
					.require_list(options_id, &causes)?
					.iter(nodes);
				for item in items {
					let Meta(object, causes) = item?.clone();
					let option_id = object.into_id(&causes)?;

					let Meta(option_ty, option_causes) = nodes
						.require_type(option_id, &causes)?
						.clone();

					match options.entry(option_ty) {
						Entry::Vacant(entry) => {
							entry.insert(option_causes);
						}
						Entry::Occupied(mut entry) => {
							entry.get_mut().merge_with(option_causes);
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
					.require_list(types_id, &causes)?
					.iter(nodes);
				for item in items {
					let Meta(object, causes) = item?.clone();
					let option_id = object.into_id(&causes)?;

					let Meta(ty, ty_causes) = nodes
						.require_type(option_id, &causes)?
						.clone();

					match types.entry(ty) {
						Entry::Vacant(entry) => {
							entry.insert(ty_causes);
						}
						Entry::Occupied(mut entry) => {
							entry.get_mut().merge_with(ty_causes);
						}
					}
				}

				match treeldr::ty::Intersection::new(types, |ty_ref| dependencies.ty(ty_ref)) {
					Ok(intersection) => treeldr::ty::Description::Intersection(intersection),
					Err(_) => treeldr::ty::Description::Empty,
				}
			}
			Description::Restriction(r) => r.build(nodes)?,
		};

		Ok(desc)
	}
}

pub trait PseudoDescription<M>: Clone + From<Description<M>> {
	fn as_standard(&self) -> Option<&Description<M>>;

	fn as_standard_mut(&mut self) -> Option<&mut Description<M>>;
}

impl<M: Clone> PseudoDescription<M> for Description<M> {
	fn as_standard(&self) -> Option<&Description<M>> {
		Some(self)
	}

	fn as_standard_mut(&mut self) -> Option<&mut Description<M>> {
		Some(self)
	}
}

#[derive(Clone)]
pub struct Definition<M, D = Description<M>> {
	/// Identifier of the type.
	id: Id,

	/// Type description.
	desc: MetaOption<D, M>,
}

impl<M, D> Definition<M, D> {
	/// Create a new type.
	///
	/// By default, a normal type is created.
	/// It can later be changed into a non-normal type as long as no properties
	/// have been defined on it.
	pub fn new(id: Id) -> Self {
		Self {
			id,
			desc: MetaOption::default(),
		}
	}

	pub fn description(&self) -> &MetaOption<D, M> {
		&self.desc
	}

	pub fn try_map<U, E>(self, f: impl FnOnce(D) -> Result<U, E>) -> Result<Definition<M, U>, E> {
		Ok(Definition {
			id: self.id,
			desc: self.desc.try_map(f)?,
		})
	}
}

impl<M, D: PseudoDescription<M>> Definition<M, D> {
	pub fn require_datatype(&self, cause: &M) -> Result<&DataType<M>, Error<M>>
	where
		M: Clone,
	{
		let because = self.desc.metadata().unwrap().clone();
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
				cause.clone(),
			)),
			None => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: None,
					found: Some(Kind::Normal),
					because,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_datatype_mut(
		&mut self,
		cause: &M,
	) -> Result<&mut DataType<M>, Error<M>>
	where
		M: Clone,
	{
		self.desc.set_once(cause.clone(), || {
			Description::Data(DataType::default()).into()
		});
		let because = self.desc.metadata().unwrap().clone();
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
				cause.clone(),
			)),
			None => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: None,
					found: Some(Kind::Normal),
					because,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	pub fn require_normal_mut(
		&mut self,
		cause: &M,
	) -> Result<&mut Normal<M>, Error<M>>
	where
		M: Clone,
	{
		self.desc
			.set_once(cause.clone(), || Description::Normal(Normal::new()).into());
		let because = self.desc.metadata().unwrap().clone();
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
				cause.clone(),
			)),
			None => Err(Error::new(
				error::TypeMismatchKind {
					id: self.id,
					expected: None,
					found: Some(Kind::Data),
					because,
				}
				.into(),
				cause.clone(),
			)),
		}
	}

	/// Declare that this type is a datatype.
	pub fn declare_datatype(&mut self, cause: M) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.desc.set_once(cause.clone(), || {
			Description::Data(DataType::default()).into()
		});
		let because = self.desc.metadata().unwrap().clone();
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
		cause: M,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		let n = self.require_normal_mut(&cause)?;
		n.declare_property(prop_ref, cause);
		Ok(())
	}

	pub fn declare_union(
		&mut self,
		options_ref: Id,
		cause: M,
	) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.desc
			.set_once(cause.clone(), || Description::Union(options_ref).into());
		let because = self.desc.metadata().unwrap().clone();
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
		cause: M,
	) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		self.desc.set_once(cause.clone(), || {
			Description::Intersection(types_ref).into()
		});
		let because = self.desc.metadata().unwrap().clone();
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
		restriction: Restriction<M>,
		cause: M,
	) -> Result<(), Error<M>>
	where
		M: Clone,
	{
		let mut restriction = Some(restriction);
		self.desc.set_once(cause.clone(), || {
			Description::Restriction(restriction.take().unwrap()).into()
		});
		match restriction {
			Some(restriction) => {
				let because = self.desc.metadata().unwrap().clone();
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
}

impl<M: Clone> Definition<M> {
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
}

impl<M: Clone + Merge> crate::Build<M> for Definition<M> {
	type Target = treeldr::ty::Definition<M>;

	fn build(
		self,
		nodes: &mut super::context::allocated::Nodes<M>,
		dependencies: crate::Dependencies<M>,
		causes: M,
	) -> Result<Self::Target, Error<M>> {
		let desc = match self.desc.unwrap() {
			Some(Meta(desc, desc_causes)) => {
				desc.build(self.id, nodes, dependencies, desc_causes)?
			}
			None => treeldr::ty::Description::Normal(treeldr::ty::Normal::new()),
		};

		Ok(treeldr::ty::Definition::new(self.id, desc, causes))
	}
}
