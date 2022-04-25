use derivative::Derivative;
use treeldr::{Causes, Ref};

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
	type Error: From<Error<F>>
		+ From<<Self::Type as ty::PseudoDescription<F>>::Error>
		+ From<<Self::Layout as layout::PseudoDescription<F>>::Error>;

	type Type: ty::PseudoDescription<F>;
	type Layout: layout::PseudoDescription<F>;
}

pub struct StandardDescriptions;

impl<F: Clone + Ord> Descriptions<F> for StandardDescriptions {
	type Error = Error<F>;

	type Type = ty::Description<F>;
	type Layout = layout::Description;
}

pub trait Build<F> {
	type Target;
	type Error;

	fn build(
		self,
		nodes: &context::AllocatedNodes<F>,
		dependencies: Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Self::Error>;
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

pub struct Dependencies<'a, F> {
	pub types: &'a [Option<treeldr::ty::Definition<F>>],
	pub properties: &'a [Option<treeldr::prop::Definition<F>>],
	pub layouts: &'a [Option<treeldr::layout::Definition<F>>],
}

impl<'a, F> Dependencies<'a, F> {
	pub fn ty(&self, ty: Ref<treeldr::ty::Definition<F>>) -> &treeldr::ty::Definition<F> {
		self.types[ty.index()].as_ref().unwrap()
	}

	pub fn property(&self, prop: Ref<treeldr::prop::Definition<F>>) -> &treeldr::prop::Definition<F> {
		self.properties[prop.index()].as_ref().unwrap()
	}

	pub fn layout(&self, layout: Ref<treeldr::layout::Definition<F>>) -> &treeldr::layout::Definition<F> {
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
	) -> Result<(), Self::Error>;

	fn relate(
		self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<F, D>,
	) -> Result<(), Self::Error>;
}
