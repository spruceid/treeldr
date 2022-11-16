use rdf_types::{Generator, VocabularyMut};
use treeldr::{vocab, BlankIdIndex, IriIndex, Id};
pub use treeldr::multiple;

pub mod context;
pub mod error;
pub mod component;
pub mod layout;
pub mod list;
pub mod node;
pub mod resource;
pub mod prop;
// pub mod rdf;
mod single;
pub mod ty;
pub mod utils;

pub use context::Context;
pub use error::Error;
pub use layout::{ParentLayout, SubLayout};
pub use list::{ListMut, ListRef};
pub use node::Node;
pub use single::Single;
pub use multiple::Multiple;

pub trait Build<M> {
	type Target;

	fn build(
		self,
		id: Id,
		metadata: M,
	) -> Result<Self::Target, Error<M>>;
}

pub trait Document<M> {
	type LocalContext;
	type Error;

	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Self::Error>;
}

pub trait ObjectToId<M> {
	fn as_id(&self) -> Option<vocab::Id>;
	
	fn as_required_id(&self, cause: &M) -> Result<vocab::Id, Error<M>>;

	fn into_required_id(self, cause: &M) -> Result<vocab::Id, Error<M>>;
}

impl<M: Clone> ObjectToId<M> for vocab::Object<M> {
	fn as_id(&self) -> Option<vocab::Id> {
		match self {
			vocab::Object::Literal(_) => None,
			vocab::Object::Iri(id) => Some(vocab::Id::Iri(*id)),
			vocab::Object::Blank(id) => Some(vocab::Id::Blank(*id)),
		}
	}

	fn as_required_id(&self, cause: &M) -> Result<vocab::Id, Error<M>> {
		match self {
			vocab::Object::Literal(lit) => Err(Error::new(
				error::LiteralUnexpected(lit.clone()).into(),
				cause.clone(),
			)),
			vocab::Object::Iri(id) => Ok(vocab::Id::Iri(*id)),
			vocab::Object::Blank(id) => Ok(vocab::Id::Blank(*id)),
		}
	}

	fn into_required_id(self, cause: &M) -> Result<vocab::Id, Error<M>> {
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
