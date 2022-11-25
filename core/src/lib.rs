use derivative::Derivative;
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
pub use ty::Type;
pub use value::Value;
pub use vocab::{BlankIdIndex, Id, IriIndex};

/// TreeLDR model.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Model<M> {
	/// Nodes.
	nodes: BTreeMap<Id, node::Definition<M>>,
}

impl<M> Model<M> {
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
						expected: TId::new(T::TYPE.id()),
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
	model: &'m Model<F>,
	value: &'t T,
}

pub trait DisplayWithModel<F> {
	fn fmt(&self, model: &Model<F>, f: &mut fmt::Formatter) -> fmt::Result;

	fn with_model<'m>(&self, model: &'m Model<F>) -> WithModel<'m, '_, Self, F> {
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
