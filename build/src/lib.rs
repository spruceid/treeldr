use derivative::Derivative;
use rdf_types::{Generator, VocabularyMut};
use treeldr::{vocab, BlankIdIndex, IriIndex, Ref};

pub mod context;
pub mod error;
pub mod layout;
pub mod list;
pub mod node;
pub mod prop;
pub mod rdf;
pub mod ty;
pub mod utils;

pub use context::Context;
pub use error::Error;
pub use layout::{ParentLayout, SubLayout};
pub use list::{ListMut, ListRef};
pub use node::Node;

pub trait Descriptions<M>: Sized {
	type Type: ty::PseudoDescription<M>;
	type Layout: layout::PseudoDescription<M>;
}

pub trait TryMap<M, E, A: Descriptions<M>, B: Descriptions<M>> {
	fn ty<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		a: A::Type,
		causes: &M,
		source: &Context<M, A>,
		context: &mut Context<M, B>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<B::Type, E>;

	fn layout<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		a: A::Layout,
		causes: &M,
		source: &Context<M, A>,
		context: &mut Context<M, B>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<B::Layout, E>;
}

pub trait Simplify<M: Clone>: Descriptions<M> {
	type Error;
	type TryMap: Default + TryMap<M, Self::Error, Self, StandardDescriptions>;
}

pub struct StandardDescriptions;

impl<M: Clone> Descriptions<M> for StandardDescriptions {
	type Type = ty::Description<M>;
	type Layout = layout::Description<M>;
}

pub trait Build<M> {
	type Target;

	fn build(
		self,
		nodes: &mut context::allocated::Nodes<M>,
		dependencies: Dependencies<M>,
		causes: M,
	) -> Result<Self::Target, Error<M>>;
}

#[derive(Derivative)]
#[derivative(
	Clone(bound = ""),
	Copy(bound = ""),
	PartialEq(bound = ""),
	Eq(bound = ""),
	Hash(bound = "")
)]
pub enum Item<M> {
	Type(Ref<treeldr::ty::Definition<M>>),
	Property(Ref<treeldr::prop::Definition<M>>),
	Layout(Ref<treeldr::layout::Definition<M>>),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct Dependencies<'a, M> {
	pub types: &'a [Option<treeldr::ty::Definition<M>>],
	pub properties: &'a [Option<treeldr::prop::Definition<M>>],
	pub layouts: &'a [Option<treeldr::layout::Definition<M>>],
}

impl<'a, M> Dependencies<'a, M> {
	pub fn ty(&self, ty: Ref<treeldr::ty::Definition<M>>) -> &treeldr::ty::Definition<M> {
		self.types[ty.index()].as_ref().unwrap()
	}

	pub fn property(
		&self,
		prop: Ref<treeldr::prop::Definition<M>>,
	) -> &treeldr::prop::Definition<M> {
		self.properties[prop.index()].as_ref().unwrap()
	}

	pub fn layout(
		&self,
		layout: Ref<treeldr::layout::Definition<M>>,
	) -> &treeldr::layout::Definition<M> {
		self.layouts[layout.index()].as_ref().unwrap()
	}
}

pub trait Document<M, D: Descriptions<M>> {
	type LocalContext;
	type Error;

	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M, D>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Self::Error>;

	fn relate<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M, D>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Self::Error>;
}

pub trait ObjectToId<M> {
	fn as_id(&self, cause: &M) -> Result<vocab::Id, Error<M>>;

	fn into_id(self, cause: &M) -> Result<vocab::Id, Error<M>>;
}

impl<M: Clone> ObjectToId<M> for vocab::Object<M> {
	fn as_id(&self, cause: &M) -> Result<vocab::Id, Error<M>> {
		match self {
			vocab::Object::Literal(lit) => Err(Error::new(
				error::LiteralUnexpected(lit.clone()).into(),
				cause.clone(),
			)),
			vocab::Object::Iri(id) => Ok(vocab::Id::Iri(*id)),
			vocab::Object::Blank(id) => Ok(vocab::Id::Blank(*id)),
		}
	}

	fn into_id(self, cause: &M) -> Result<vocab::Id, Error<M>> {
		match self {
			vocab::Object::Literal(lit) => Err(Error::new(
				error::LiteralUnexpected(lit).into(),
				cause.clone(),
			)),
			vocab::Object::Iri(id) => Ok(vocab::Id::Iri(id)),
			vocab::Object::Blank(id) => Ok(vocab::Id::Blank(id)),
		}
	}
}
