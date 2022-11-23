use iref::IriRefBuf;
use locspan::Meta;
use rdf_types::{Generator, VocabularyMut};
use thiserror::Error;
use treeldr::{metadata::Merge, vocab::*, Id};
use treeldr_build::Context;

mod local_context;
mod error;
mod ty;
mod prop;
mod layout;

pub use local_context::LocalContext;
pub use error::{Error, LocalError};

impl<M: Clone + Merge> treeldr_build::Document<M> for crate::Document<M> {
	type LocalContext = LocalContext<M>;
	type Error = Error<M>;

	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let mut declared_base_iri = None;
		for Meta(base_iri, loc) in &self.bases {
			match declared_base_iri.take() {
				Some(Meta(declared_base_iri, d_loc)) => {
					if declared_base_iri != *base_iri {
						return Err(Meta(
							LocalError::BaseIriMismatch {
								expected: Box::new(declared_base_iri),
								found: Box::new(base_iri.clone()),
								because: d_loc,
							},
							loc.clone(),
						)
						.into());
					}
				}
				None => {
					local_context.set_base_iri(base_iri.clone());
					declared_base_iri = Some(Meta(base_iri.clone(), loc.clone()));
				}
			}
		}

		for import in &self.uses {
			import.declare(local_context, context, vocabulary, generator)?
		}

		for ty in &self.types {
			ty.declare(local_context, context, vocabulary, generator)?
		}

		for prop in &self.properties {
			prop.declare(local_context, context, vocabulary, generator)?
		}

		for layout in &self.layouts {
			layout.declare(local_context, context, vocabulary, generator)?
		}

		Ok(())
	}

	fn define<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		for ty in self.types {
			ty.build(local_context, context, vocabulary, generator)?
		}

		for prop in self.properties {
			prop.build(local_context, context, vocabulary, generator)?;
		}

		for layout in self.layouts {
			layout.build(local_context, context, vocabulary, generator)?
		}

		Ok(())
	}
}

pub trait Declare<M: Clone + Merge> {
	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>>;
}

pub trait Build<M: Clone + Merge> {
	type Target;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>>;
}

impl<M: Clone + Merge> Build<M> for Meta<crate::Documentation<M>, M> {
	type Target = Option<Meta<String, M>>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		_local_context: &mut LocalContext<M>,
		_context: &mut Context<M>,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(doc, loc) = self;

		let mut description = String::new();

		for Meta(line, _) in doc.items {
			if !description.is_empty() {
				description.push('\n');
			}

			description.push_str(&line);
		}


		let description = if description.is_empty() {
			None
		} else {
			Some(Meta(description, loc))
		};

		Ok(description)
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::Id, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		_context: &mut Context<M>,
		vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(id, loc) = self;
		let iri = match id {
			crate::Id::Name(name) => {
				let mut iri_ref = IriRefBuf::from_string(name).unwrap();
				iri_ref.resolve(local_context.base_iri(vocabulary, loc.clone())?.as_iri());
				iri_ref.try_into().unwrap()
			}
			crate::Id::IriRef(iri_ref) => {
				iri_ref.resolved(local_context.base_iri(vocabulary, loc.clone())?.as_iri())
			}
			crate::Id::Compact(prefix, iri_ref) => {
				local_context.expand_compact_iri(&prefix, iri_ref.as_iri_ref(), &loc)?
			}
		};

		Ok(Meta(Id::Iri(vocabulary.insert(iri.as_iri())), loc))
	}
}

impl<M: Clone + Merge> Declare<M> for Meta<crate::Use<M>, M> {
	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut LocalContext<M>,
		_context: &mut Context<M>,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		local_context
			.declare_prefix(
				self.prefix.value().as_str().to_string(),
				self.iri.value().clone(),
				self.metadata().clone(),
			)
			.map_err(Into::into)
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::Alias, M> {
	type Target = Meta<treeldr::Name, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		_local_context: &mut LocalContext<M>,
		_context: &mut Context<M>,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(name, loc) = self;
		match treeldr::Name::new(name.as_str()) {
			Ok(name) => Ok(Meta(name, loc)),
			Err(_) => Err(treeldr_build::Error::new(
				treeldr_build::error::NameInvalid(name.into_string()).into(),
				loc,
			)
			.into()),
		}
	}
}