use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
pub use treeldr::{multiple, property_values};
use treeldr::{vocab, BlankIdIndex, IriIndex, Value};

pub mod component;
pub mod context;
pub mod error;
pub mod functional_property_value;
pub mod layout;
pub mod list;
pub mod prop;
pub mod rdf;
pub mod resource;
pub mod single;
pub mod ty;
pub mod utils;

pub use context::Context;
pub use error::Error;
pub use functional_property_value::FunctionalPropertyValue;
pub use layout::{ParentLayout, SubLayout};
pub use list::{ListMut, ListRef};
pub use multiple::Multiple;
pub use prop::Property;
pub use property_values::{PropertyValue, PropertyValueRef, PropertyValues};
pub use single::Single;
pub use ty::Type;

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

pub trait MetaValueExt<M>: Sized {
	fn into_expected_id(self) -> Result<Meta<vocab::Id, M>, Error<M>>;

	fn into_expected_literal(self) -> Result<Meta<treeldr::value::Literal, M>, Error<M>>;

	fn into_expected_type(self) -> Result<Meta<Type, M>, Error<M>> {
		Ok(self.into_expected_id()?.map(Type::from))
	}

	fn into_expected_numeric(self) -> Result<Meta<treeldr::value::Numeric, M>, Error<M>> {
		match self.into_expected_literal()? {
			Meta(treeldr::value::Literal::Numeric(n), meta) => Ok(Meta(n, meta)),
			_ => panic!("expected number"),
		}
	}

	fn into_expected_integer(self) -> Result<Meta<treeldr::value::Integer, M>, Error<M>> {
		match self.into_expected_literal()? {
			Meta(treeldr::value::Literal::Numeric(n), meta) => match n.into_integer() {
				Ok(i) => Ok(Meta(i, meta)),
				Err(_) => panic!("expected integer"),
			},
			_ => panic!("expected integer"),
		}
	}

	fn into_expected_non_negative_integer(
		self,
	) -> Result<Meta<treeldr::value::NonNegativeInteger, M>, Error<M>> {
		match self.into_expected_literal()? {
			Meta(treeldr::value::Literal::Numeric(n), meta) => {
				match n.into_non_negative_integer() {
					Ok(n) => Ok(Meta(n, meta)),
					Err(_) => panic!("expected non negative integer"),
				}
			}
			_ => panic!("expected non negative integer"),
		}
	}

	fn into_expected_regexp(self) -> Result<Meta<treeldr::ty::data::RegExp, M>, Error<M>> {
		match self.into_expected_literal()? {
			Meta(treeldr::value::Literal::Other(_, _), _meta) => {
				todo!("regexp")
			}
			_ => panic!("expected regular expression"),
		}
	}

	fn into_expected_schema_boolean(self) -> Result<Meta<bool, M>, Error<M>> {
		match self.into_expected_id()? {
			Meta(
				treeldr::Id::Iri(IriIndex::Iri(vocab::Term::Schema(vocab::Schema::True))),
				meta,
			) => Ok(Meta(true, meta)),
			Meta(
				treeldr::Id::Iri(IriIndex::Iri(vocab::Term::Schema(vocab::Schema::False))),
				meta,
			) => Ok(Meta(false, meta)),
			_ => panic!("expected boolean"),
		}
	}

	fn into_expected_name(self) -> Result<Meta<treeldr::Name, M>, Error<M>> {
		match self.into_expected_literal()? {
			Meta(treeldr::value::Literal::String(s), meta) => match treeldr::Name::new(s) {
				Ok(name) => Ok(Meta(name, meta)),
				Err(_) => panic!("invalid name"),
			},
			_ => panic!("expected regular expression"),
		}
	}
}

impl<M> MetaValueExt<M> for Meta<Value, M> {
	fn into_expected_id(self) -> Result<Meta<vocab::Id, M>, Error<M>> {
		match self {
			Meta(Value::Node(id), meta) => Ok(Meta(id, meta)),
			Meta(Value::Literal(lit), meta) => {
				Err(Error::new(error::LiteralUnexpected(lit).into(), meta))
			}
		}
	}

	fn into_expected_literal(self) -> Result<Meta<treeldr::value::Literal, M>, Error<M>> {
		match self {
			Meta(Value::Node(id), meta) => Err(Error::new(error::LiteralExpected(id).into(), meta)),
			Meta(Value::Literal(lit), meta) => Ok(Meta(lit, meta)),
		}
	}
}
