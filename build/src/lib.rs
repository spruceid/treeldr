use derivative::Derivative;
use locspan::Location;
use treeldr::{vocab::Name, Caused, Causes, Id, Ref, Vocabulary, WithCauses};

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
pub use error::{Error, ErrorWithVocabulary};
pub use layout::{ParentLayout, SubLayout};
pub use list::{ListMut, ListRef};
pub use node::Node;
pub use rdf::GrdfDefinitions;

pub trait Definitions<F>: Sized {
	type Error: From<Error<F>> + From<<Self::Type as Build<F>>::Error> + From<<Self::Property as Build<F>>::Error> + From<<Self::Layout as Build<F>>::Error>;

	type Type: Build<F, Target = treeldr::ty::Definition<F>>;
	type Property: Build<F, Target = treeldr::prop::Definition<F>>;
	type Layout: Layout<F, Self>;
}

pub trait Layout<F, D: Definitions<F>>: Build<F, Target = treeldr::layout::Definition<F>> {
	/// Computes the list of sub layouts used by this layout.
	fn sub_layouts(&self, context: &Context<F, D>) -> Result<Vec<SubLayout<F>>, Error<F>>;

	fn name(&self) -> Option<&WithCauses<Name, F>>;

	fn set_name(&mut self, name: Name, cause: Option<Location<F>>) -> Result<(), Error<F>>;

	/// Computes a default name for the layout.
	fn default_name(
		&self,
		context: &Context<F, D>,
		parent_layouts: &[WithCauses<ParentLayout, F>],
		cause: Option<Location<F>>,
	) -> Result<Option<Caused<Name, F>>, Error<F>>;
}

pub trait Build<F> {
	type Target;
	type Error;

	fn dependencies(
		&self,
		_id: Id,
		_nodes: &context::AllocatedNodes<F>,
		_causes: &Causes<F>,
	) -> Result<Vec<Item<F>>, Self::Error> {
		Ok(Vec::new())
	}

	fn build(
		self,
		id: Id,
		vocab: &Vocabulary,
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

pub trait Document<F, D: Definitions<F>> {
	type LocalContext;
	type Error;

	fn declare(&self, local_context: &mut Self::LocalContext, context: &mut Context<F, D>) -> Result<(), Self::Error>;

	fn relate(self, local_context: &mut Self::LocalContext, context: &mut Context<F, D>) -> Result<(), Self::Error>;
}
