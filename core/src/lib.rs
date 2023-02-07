use derivative::Derivative;
use metadata::Merge;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub mod component;
pub mod doc;
pub mod error;
mod feature;
pub mod layout;
pub mod list;
mod meta_option;
pub mod metadata;
pub mod multiple;
pub mod name;
pub mod node;
pub mod prop;
pub mod property_values;
pub mod reporting;
pub mod to_rdf;
pub mod ty;
pub mod utils;
pub mod value;
pub mod vocab;

pub use doc::{Documentation, StrippedDocumentation};
pub use error::Error;
pub use feature::Feature;
pub use layout::Layout;
pub use meta_option::MetaOption;
pub use metadata::Metadata;
pub use multiple::Multiple;
pub use name::Name;
pub use prop::Property;
pub use property_values::{
	FunctionalPropertyValue, PropertyValue, PropertyValueRef, PropertyValues,
	RequiredFunctionalPropertyValue,
};
pub use ty::Type;
pub use value::Value;
pub use vocab::{BlankIdIndex, Id, IriIndex};

/// TreeLDR model.
pub struct Model<M> {
	/// Inner mutable model.
	inner: MutableModel<M>,

	/// Type hierarchy.
	type_hierarchy: ty::Hierarchy<M>,

	/// Type dependencies.
	type_dependencies: ty::Dependencies<M>,

	/// Type properties.
	type_properties: HashMap<TId<Type>, ty::Properties<M>>,
}

impl<M> Model<M> {
	pub fn new(model: MutableModel<M>) -> Result<Self, ty::restriction::Contradiction>
	where
		M: Clone + Merge,
	{
		let type_hierarchy = ty::Hierarchy::new(&model);
		let type_dependencies = ty::Dependencies::new(&model);
		let type_properties = type_dependencies.compute_class_properties(&model)?;

		Ok(Self {
			inner: model,
			type_hierarchy,
			type_dependencies,
			type_properties,
		})
	}

	pub fn type_hierarchy(&self) -> &ty::Hierarchy<M> {
		&self.type_hierarchy
	}

	pub fn type_dependencies(&self) -> &ty::Dependencies<M> {
		&self.type_dependencies
	}

	pub fn type_properties(&self, ty: TId<Type>) -> Option<&ty::Properties<M>> {
		self.type_properties.get(&ty)
	}
}

impl<M> Deref for Model<M> {
	type Target = MutableModel<M>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<M> Borrow<MutableModel<M>> for Model<M> {
	fn borrow(&self) -> &MutableModel<M> {
		&self.inner
	}
}

impl<M> AsRef<MutableModel<M>> for Model<M> {
	fn as_ref(&self) -> &MutableModel<M> {
		&self.inner
	}
}

/// TreeLDR mutable model.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct MutableModel<M> {
	/// Nodes.
	nodes: BTreeMap<Id, node::Definition<M>>,
}

impl<M> MutableModel<M> {
	/// Creates a new empty context.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_parts(nodes: BTreeMap<Id, node::Definition<M>>) -> Self {
		Self { nodes }
	}

	pub fn can_be_reference_layout(
		&self,
		map: &mut HashMap<TId<Layout>, bool>,
		r: TId<Layout>,
	) -> bool {
		match map.get(&r).cloned() {
			Some(b) => b,
			None => {
				let b = self.get(r).unwrap().as_layout().can_be_reference(map, self);
				map.insert(r, b);
				b
			}
		}
	}

	/// Returns the node associated to the given `id`, if any.
	pub fn get_resource(&self, id: Id) -> Option<Ref<node::Resource, M>> {
		self.get(TId::new(id))
	}

	/// Returns a mutable reference to the node associated to the given `id`, if any.
	pub fn get_resource_mut(&mut self, id: Id) -> Option<RefMut<node::Resource, M>> {
		self.get_mut(TId::new(id))
	}

	/// Returns the node associated to the given `id`, if any.
	pub fn get<T: ResourceType>(&self, id: TId<T>) -> Option<Ref<T, M>> {
		self.nodes.get(&id.0).and_then(|n| {
			if T::check(n) {
				Some(Ref(n, PhantomData))
			} else {
				None
			}
		})
	}

	/// Returns a mutable reference to the node associated to the given `id`, if any.
	pub fn get_mut<T: ResourceType>(&mut self, id: TId<T>) -> Option<RefMut<T, M>> {
		self.nodes.get_mut(&id.0).and_then(|n| {
			if T::check(n) {
				Some(RefMut(n, PhantomData))
			} else {
				None
			}
		})
	}

	pub fn nodes(&self) -> impl Iterator<Item = (Id, &node::Definition<M>)> {
		self.nodes.iter().map(|(i, n)| (*i, n))
	}

	pub fn nodes_mut(&mut self) -> impl Iterator<Item = (Id, &mut node::Definition<M>)> {
		self.nodes.iter_mut().map(|(i, n)| (*i, n))
	}

	pub fn classes(&self) -> impl Iterator<Item = (TId<Type>, Ref<Type, M>)> {
		self.nodes.iter().filter_map(|(i, n)| {
			if n.is_type() {
				Some((TId(*i, PhantomData), Ref(n, PhantomData)))
			} else {
				None
			}
		})
	}

	pub fn properties(&self) -> impl Iterator<Item = (TId<Property>, Ref<Property, M>)> {
		self.nodes.iter().filter_map(|(i, n)| {
			if n.is_property() {
				Some((TId(*i, PhantomData), Ref(n, PhantomData)))
			} else {
				None
			}
		})
	}

	pub fn layouts(&self) -> impl Iterator<Item = (TId<Layout>, Ref<Layout, M>)> {
		self.nodes.iter().filter_map(|(i, n)| {
			if n.is_layout() {
				Some((TId(*i, PhantomData), Ref(n, PhantomData)))
			} else {
				None
			}
		})
	}

	/// Inserts the given node to the context.
	///
	/// Replaces any previous node with the same [`node::Definition::id`].
	pub fn insert(&mut self, node: node::Definition<M>) -> Option<node::Definition<M>> {
		self.nodes.insert(node.id(), node)
	}

	pub fn require<T: ResourceType>(&self, id: TId<T>) -> Result<Ref<T, M>, Error<M>>
	where
		M: Clone,
	{
		match self.nodes.get(&id.0) {
			Some(r) => {
				if T::check(r) {
					Ok(Ref(r, PhantomData))
				} else {
					Err(error::NodeInvalidType {
						id: id.id(),
						expected: T::TYPE.id(),
						found: r.type_().clone(),
					}
					.into())
				}
			}
			None => Err(error::NodeUnknown {
				id: id.id(),
				expected_ty: T::TYPE,
			}
			.into()),
		}
	}
}

pub struct WithModel<'m, 't, T: ?Sized, F> {
	model: &'m MutableModel<F>,
	value: &'t T,
}

pub trait DisplayWithModel<F> {
	fn fmt(&self, model: &MutableModel<F>, f: &mut fmt::Formatter) -> fmt::Result;

	fn with_model<'m>(&self, model: &'m MutableModel<F>) -> WithModel<'m, '_, Self, F> {
		WithModel { model, value: self }
	}
}

impl<'m, 't, T: DisplayWithModel<F>, F> fmt::Display for WithModel<'m, 't, T, F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.value.fmt(self.model, f)
	}
}

pub trait ResourceType {
	const TYPE: Type;

	fn check<M>(resource: &node::Definition<M>) -> bool;
}

/// Typed identifier.
#[derive(Derivative)]
#[derivative(
	Debug(bound = ""),
	Clone(bound = ""),
	Copy(bound = ""),
	PartialEq(bound = ""),
	Eq(bound = ""),
	PartialOrd(bound = ""),
	Ord(bound = ""),
	Hash(bound = "")
)]
pub struct TId<T>(Id, PhantomData<T>);

impl<T> TId<T> {
	pub fn new(id: Id) -> Self {
		Self(id, PhantomData)
	}

	pub fn id(&self) -> Id {
		self.0
	}

	pub fn into_id(self) -> Id {
		self.0
	}
}

impl<T, V> contextual::DisplayWithContext<V> for TId<T>
where
	Id: contextual::DisplayWithContext<V>,
{
	fn fmt_with(&self, context: &V, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt_with(context, f)
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
/// Typed Resource reference.
pub struct Ref<'a, T, M>(&'a node::Definition<M>, PhantomData<T>);

impl<'a, T, M> Ref<'a, T, M> {
	pub fn as_resource(&self) -> &'a node::Definition<M> {
		self.0
	}

	pub fn into_resource(self) -> &'a node::Definition<M> {
		self.0
	}
}

impl<'a, T, M> Deref for Ref<'a, T, M> {
	type Target = node::Definition<M>;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

pub struct RefMut<'a, T, M>(&'a mut node::Definition<M>, PhantomData<T>);

impl<'a, T, M> RefMut<'a, T, M> {
	pub fn as_resource(&self) -> &node::Definition<M> {
		self.0
	}

	pub fn into_resource(self) -> &'a node::Definition<M> {
		self.0
	}

	pub fn as_resource_mut(&mut self) -> &mut node::Definition<M> {
		self.0
	}

	pub fn into_resource_mut(self) -> &'a mut node::Definition<M> {
		self.0
	}
}

impl<'a, T, M> Deref for RefMut<'a, T, M> {
	type Target = node::Definition<M>;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<'a, T, M> DerefMut for RefMut<'a, T, M> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0
	}
}
