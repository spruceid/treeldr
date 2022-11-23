use rdf_types::{Generator, VocabularyMut};
use treeldr::{vocab, BlankIdIndex, IriIndex};
pub use treeldr::multiple;

pub mod context;
pub mod error;
pub mod component;
pub mod layout;
pub mod list;
pub mod resource;
pub mod prop;
pub mod rdf;
mod single;
pub mod ty;
pub mod utils;

pub use ty::Type;
pub use prop::Property;
pub use context::Context;
pub use error::Error;
pub use layout::{ParentLayout, SubLayout};
pub use list::{ListMut, ListRef};
pub use single::Single;
pub use multiple::Multiple;

pub trait Document<M> {
	type LocalContext;
	type Error;

	/// Declare in `context` all the node declared in the document.
	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Self::Error>;

	/// Define in `context` all the statements of the document.
	fn define<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Self::Error>;
}

pub trait ObjectAsId<M> {
	fn as_id(&self) -> Option<vocab::Id>;
}

impl<M> ObjectAsId<M> for vocab::Object<M> {
	fn as_id(&self) -> Option<vocab::Id> {
		match self {
			vocab::Object::Literal(_) => None,
			vocab::Object::Iri(id) => Some(vocab::Id::Iri(*id)),
			vocab::Object::Blank(id) => Some(vocab::Id::Blank(*id)),
		}
	}
}

pub trait ObjectAsRequiredId<M> {
	fn as_required_id(&self, cause: &M) -> Result<vocab::Id, Error<M>>;

	fn into_required_id(self, cause: &M) -> Result<vocab::Id, Error<M>>;
}

impl<M: Clone> ObjectAsRequiredId<M> for vocab::Object<M> {
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
