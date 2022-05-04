use derivative::Derivative;
use treeldr::{vocab, Causes, Ref, Vocabulary};

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

pub trait Descriptions<F>: Sized {
	type Type: ty::PseudoDescription<F>;
	type Layout: layout::PseudoDescription<F>;
}

pub trait TryMap<F, E, A: Descriptions<F>, B: Descriptions<F>> {
	fn ty(
		&self,
		a: A::Type,
		causes: &Causes<F>,
		source: &Context<F, A>,
		context: &mut Context<F, B>,
		vocabulary: &mut Vocabulary,
	) -> Result<B::Type, E>;
	fn layout(
		&self,
		a: A::Layout,
		causes: &Causes<F>,
		source: &Context<F, A>,
		context: &mut Context<F, B>,
		vocabulary: &mut Vocabulary,
	) -> Result<B::Layout, E>;
}

pub trait Simplify<F: Clone + Ord>: Descriptions<F> {
	type Error;
	type TryMap: Default + TryMap<F, Self::Error, Self, StandardDescriptions>;
}

pub struct StandardDescriptions;

impl<F: Clone + Ord> Descriptions<F> for StandardDescriptions {
	type Type = ty::Description<F>;
	type Layout = layout::Description<F>;
}

pub trait Build<F> {
	type Target;

	fn build(
		self,
		nodes: &mut context::allocated::Nodes<F>,
		dependencies: Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Error<F>>;
}

#[derive(Derivative)]
#[derivative(
	Clone(bound = ""),
	Copy(bound = ""),
	PartialEq(bound = ""),
	Eq(bound = ""),
	Hash(bound = "")
)]
pub enum Item<F> {
	Type(Ref<treeldr::ty::Definition<F>>),
	Property(Ref<treeldr::prop::Definition<F>>),
	Layout(Ref<treeldr::layout::Definition<F>>),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct Dependencies<'a, F> {
	pub types: &'a [Option<treeldr::ty::Definition<F>>],
	pub properties: &'a [Option<treeldr::prop::Definition<F>>],
	pub layouts: &'a [Option<treeldr::layout::Definition<F>>],
}

impl<'a, F> Dependencies<'a, F> {
	pub fn ty(&self, ty: Ref<treeldr::ty::Definition<F>>) -> &treeldr::ty::Definition<F> {
		self.types[ty.index()].as_ref().unwrap()
	}

	pub fn property(
		&self,
		prop: Ref<treeldr::prop::Definition<F>>,
	) -> &treeldr::prop::Definition<F> {
		self.properties[prop.index()].as_ref().unwrap()
	}

	pub fn layout(
		&self,
		layout: Ref<treeldr::layout::Definition<F>>,
	) -> &treeldr::layout::Definition<F> {
		self.layouts[layout.index()].as_ref().unwrap()
	}
}

pub trait Document<F, D: Descriptions<F>> {
	type LocalContext;
	type Error;

	fn declare(
		&self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<F, D>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Self::Error>;

	fn relate(
		self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<F, D>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Self::Error>;
}

pub trait ObjectToId<F> {
	fn as_id(&self, cause: Option<&locspan::Location<F>>) -> Result<vocab::Id, Error<F>>;

	fn into_id(self, cause: Option<&locspan::Location<F>>) -> Result<vocab::Id, Error<F>>;
}

impl<F: Clone> ObjectToId<F> for vocab::Object<F> {
	fn as_id(&self, cause: Option<&locspan::Location<F>>) -> Result<vocab::Id, Error<F>> {
		match self {
			vocab::Object::Literal(lit) => Err(Error::new(
				error::LiteralUnexpected(lit.clone()).into(),
				cause.cloned(),
			)),
			vocab::Object::Iri(id) => Ok(vocab::Id::Iri(*id)),
			vocab::Object::Blank(id) => Ok(vocab::Id::Blank(*id)),
		}
	}

	fn into_id(self, cause: Option<&locspan::Location<F>>) -> Result<vocab::Id, Error<F>> {
		match self {
			vocab::Object::Literal(lit) => Err(Error::new(
				error::LiteralUnexpected(lit).into(),
				cause.cloned(),
			)),
			vocab::Object::Iri(id) => Ok(vocab::Id::Iri(id)),
			vocab::Object::Blank(id) => Ok(vocab::Id::Blank(id)),
		}
	}
}
