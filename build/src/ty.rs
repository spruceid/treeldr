use crate::{error, utils::TryCollect, Error};
use derivative::Derivative;
use locspan::Location;
use std::collections::{BTreeMap, HashMap};
use treeldr::{vocab, Caused, Causes, Id, MaybeSet, WithCauses};

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

	/// Property restriction.
	Restriction(Restriction<F>),
}

impl<F: Clone + Ord> Description<F> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
			Self::Intersection(_) => Kind::Intersection,
			Self::Restriction(_) => Kind::Restriction,
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
			_ => None,
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
		_id: Id,
		nodes: &super::context::AllocatedNodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<treeldr::ty::Description<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		let desc = match self {
			Self::Normal(n) => n.build(nodes)?,
			Self::Union(options_id) => {
				use std::collections::btree_map::Entry;
				let mut options = BTreeMap::new();

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
}

impl<F, D: PseudoDescription<F>> crate::Build<F> for Definition<F, D> {
	type Target = treeldr::ty::Definition<F>;
	type Error = D::Error;

	fn build(
		self,
		_vocab: &mut treeldr::Vocabulary,
		nodes: &mut super::context::AllocatedNodes<F>,
		_additional: &mut crate::AdditionalNodes<F>,
		dependencies: crate::Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Self::Error> {
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RangeRestriction {
	Any(Id),
	All(Id),
}

pub type CardinalityRestriction = treeldr::prop::restriction::Cardinality;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyRestriction {
	Range(RangeRestriction),
	Cardinality(CardinalityRestriction),
}

pub struct Restriction<F> {
	property: WithCauses<Id, F>,
	restrictions: BTreeMap<PropertyRestriction, Causes<F>>,
}

impl<F> PartialEq for Restriction<F> {
	fn eq(&self, other: &Self) -> bool {
		self.property.inner() == other.property.inner()
			&& self.restrictions.len() == other.restrictions.len()
			&& self
				.restrictions
				.keys()
				.zip(other.restrictions.keys())
				.all(|(a, b)| a == b)
	}
}

impl<F> Restriction<F> {
	pub fn new(property: WithCauses<Id, F>) -> Self {
		Self {
			property,
			restrictions: BTreeMap::new(),
		}
	}

	pub fn add_restriction(&mut self, r: PropertyRestriction, cause: Option<Location<F>>)
	where
		F: Ord,
	{
		use std::collections::btree_map::Entry;
		match self.restrictions.entry(r) {
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
		let (prop_id, prop_causes) = self.property.into_parts();
		let prop_ref = nodes.require_property(prop_id, prop_causes.preferred().cloned())?;

		let mut restrictions = treeldr::prop::Restrictions::new();
		for (restriction, restriction_causes) in self.restrictions {
			if let Err(treeldr::prop::restriction::Contradiction) =
				restrictions.restrict(restriction.build(nodes, &restriction_causes)?)
			{
				return Ok(treeldr::ty::Description::Empty);
			}
		}

		let result =
			treeldr::ty::Restriction::new(WithCauses::new(**prop_ref, prop_causes), restrictions);
		Ok(treeldr::ty::Description::Restriction(result))
	}
}

impl PropertyRestriction {
	pub fn build<F>(
		self,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<treeldr::prop::Restriction<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		match self {
			Self::Range(r) => Ok(treeldr::prop::Restriction::Range(r.build(nodes, causes)?)),
			Self::Cardinality(c) => Ok(treeldr::prop::Restriction::Cardinality(c)),
		}
	}
}

impl RangeRestriction {
	pub fn build<F>(
		self,
		nodes: &super::context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<treeldr::prop::restriction::Range<F>, Error<F>>
	where
		F: Clone + Ord,
	{
		match self {
			Self::Any(id) => {
				let ty_ref = nodes.require_type(id, causes.preferred().cloned())?;
				Ok(treeldr::prop::restriction::Range::Any(**ty_ref))
			}
			Self::All(id) => {
				let ty_ref = nodes.require_type(id, causes.preferred().cloned())?;
				Ok(treeldr::prop::restriction::Range::All(**ty_ref))
			}
		}
	}
}
