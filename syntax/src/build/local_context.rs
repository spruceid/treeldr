use std::collections::HashMap;

use iref::{IriBuf, IriRef};
use locspan::Meta;
use rdf_types::{Generator, IriVocabulary, VocabularyMut};
use treeldr::{
	metadata::Merge,
	vocab::{Term, TldrVocabulary, Xsd},
	BlankIdIndex, Id, IriIndex,
};
use treeldr_build::Context;

use super::{Error, LocalError};

/// Build context.
pub struct LocalContext<M> {
	/// Base IRI of th parsed document.
	base_iri: Option<IriBuf>,

	/// Bound prefixes.
	prefixes: HashMap<String, Meta<IriBuf, M>>,

	/// Current scope.
	pub(crate) scope: Option<Id>,

	pub(crate) alias_id: Option<Meta<Id, M>>,

	label_id: HashMap<crate::Label, Meta<Id, M>>,

	/// Flag indicating if the (layout) definition is implicit.
	///
	/// If `true`, then the layout will be bound to itself.
	pub(crate) implicit_definition: bool,
}

impl<M> LocalContext<M> {
	pub fn new(base_iri: Option<IriBuf>) -> Self {
		Self {
			base_iri,
			prefixes: HashMap::new(),
			scope: None,
			alias_id: None,
			label_id: HashMap::new(),
			implicit_definition: false,
		}
	}

	pub fn anonymous_id<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		label: Option<crate::Label>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		loc: M,
	) -> Meta<Id, M>
	where
		M: Clone,
	{
		let alias_id = self.alias_id.take();
		let id = match label {
			Some(label) => {
				use std::collections::hash_map::Entry;
				match self.label_id.entry(label) {
					Entry::Occupied(entry) => entry.get().clone(),
					Entry::Vacant(entry) => {
						let id = alias_id.unwrap_or_else(|| Meta(generator.next(vocabulary), loc));
						entry.insert(id.clone());
						id
					}
				}
			}
			None => alias_id.unwrap_or_else(|| Meta(generator.next(vocabulary), loc)),
		};

		self.alias_id.take();
		id
	}
}

impl<M: Clone> LocalContext<M> {
	pub fn base_iri(
		&self,
		vocabulary: &TldrVocabulary,
		loc: M,
	) -> Result<IriBuf, Meta<LocalError<M>, M>> {
		match &self.scope {
			Some(Id::Iri(scope)) => {
				let mut iri = vocabulary.iri(scope).unwrap().to_owned();
				iri.path_mut().open();
				Ok(iri)
			}
			_ => self
				.base_iri
				.clone()
				.ok_or(Meta(LocalError::NoBaseIri, loc)),
		}
	}

	pub fn set_base_iri(&mut self, base_iri: IriBuf) {
		self.base_iri = Some(base_iri)
	}

	pub fn declare_prefix(
		&mut self,
		prefix: String,
		iri: IriBuf,
		loc: M,
	) -> Result<(), Meta<LocalError<M>, M>> {
		use std::collections::hash_map::Entry;
		match self.prefixes.entry(prefix) {
			Entry::Occupied(entry) => Err(Meta(
				LocalError::AlreadyDefinedPrefix(
					entry.key().to_owned(),
					entry.get().metadata().clone(),
				),
				loc,
			)),
			Entry::Vacant(entry) => {
				entry.insert(Meta(iri, loc));
				Ok(())
			}
		}
	}

	pub fn expand_compact_iri(
		&self,
		prefix: &str,
		iri_ref: IriRef,
		loc: &M,
	) -> Result<IriBuf, Meta<LocalError<M>, M>> {
		match self.prefixes.get(prefix) {
			Some(iri) => match IriBuf::try_from(iri.as_str().to_string() + iri_ref.as_str()) {
				Ok(iri) => Ok(iri),
				Err((_, string)) => Err(Meta(
					LocalError::InvalidExpandedCompactIri(string),
					loc.clone(),
				)),
			},
			None => Err(Meta(
				LocalError::UndefinedPrefix(prefix.to_owned()),
				loc.clone(),
			)),
		}
	}

	/// Generate a literal type with the given `id`.
	pub fn generate_literal_type(
		&self,
		Meta(id, _): &Meta<Id, M>,
		context: &mut Context<M>,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
		Meta(lit, loc): Meta<crate::Literal, M>,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		if id.is_blank() {
			// Declare the type.
			context.declare_datatype(*id, loc.clone());
		}

		if self.implicit_definition {
			context
				.get_mut(*id)
				.unwrap()
				.as_layout_mut()
				.ty_mut()
				.insert_base(Meta(*id, loc.clone()))
		}

		let regexp = match lit {
			crate::Literal::String(s) => treeldr::ty::data::RegExp::from(s),
			crate::Literal::RegExp(regexp_string) => {
				match treeldr::ty::data::RegExp::parse(&regexp_string) {
					Ok(regexp) => regexp,
					Err(e) => {
						return Err(treeldr_build::Error::new(
							treeldr_build::error::RegExpInvalid(regexp_string, e).into(),
							loc,
						)
						.into())
					}
				}
			}
		};

		use treeldr_build::ty::datatype::{restriction, Restriction};
		let restriction_id = generator.next(vocabulary);
		let restriction = context.declare_datatype_restriction(restriction_id, loc.clone());
		restriction
			.as_datatype_restriction_mut()
			.restriction_mut()
			.insert_base(Meta(
				Restriction::String(restriction::String::Pattern(regexp)),
				loc.clone(),
			));
		let restrictions_list = context.create_list(
			vocabulary,
			generator,
			Some(Meta(restriction_id.into_term(), loc.clone())),
		);

		let ty = context.get_mut(*id).unwrap().as_type_mut();
		let dt = ty.as_datatype_mut();
		dt.base_mut().insert_base(Meta(
			Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::String))),
			loc.clone(),
		));
		dt.restrictions_mut()
			.insert_base(Meta(restrictions_list, loc));

		Ok(())
	}

	/// Generate a literal layout with the given `id`.
	pub fn generate_literal_layout(
		&self,
		Meta(id, _): &Meta<Id, M>,
		context: &mut Context<M>,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
		Meta(lit, loc): Meta<crate::Literal, M>,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		if id.is_blank() {
			// Define the associated layout.
			context.declare_layout(*id, loc.clone());
		}

		if self.implicit_definition {
			context
				.get_mut(*id)
				.unwrap()
				.as_layout_mut()
				.ty_mut()
				.insert_base(Meta(*id, loc.clone()));
		}

		let regexp = match lit {
			crate::Literal::String(s) => treeldr::ty::data::RegExp::from(s),
			crate::Literal::RegExp(regexp_string) => {
				match treeldr::ty::data::RegExp::parse(&regexp_string) {
					Ok(regexp) => regexp,
					Err(e) => {
						return Err(treeldr_build::Error::new(
							treeldr_build::error::RegExpInvalid(regexp_string, e).into(),
							loc,
						)
						.into())
					}
				}
			}
		};

		use treeldr_build::layout::{restriction, Restriction};
		let restriction_id = generator.next(vocabulary);
		let restriction = context.declare_layout_restriction(restriction_id, loc.clone());
		restriction
			.as_layout_restriction_mut()
			.restriction_mut()
			.insert_base(Meta(
				Restriction::Primitive(restriction::primitive::Restriction::String(
					restriction::primitive::String::Pattern(regexp),
				)),
				loc.clone(),
			));
		let restrictions_list = context.create_list(
			vocabulary,
			generator,
			Some(Meta(restriction_id.into_term(), loc.clone())),
		);

		let layout = context.get_mut(*id).unwrap().as_layout_mut();
		layout.description_mut().insert_base(Meta(
			treeldr_build::layout::BaseDescriptionBinding::DerivedFrom(
				treeldr_build::layout::Primitive::String.id(),
			),
			loc.clone(),
		));
		layout
			.restrictions_mut()
			.insert_base(Meta(restrictions_list, loc));

		Ok(())
	}

	// /// Inserts a new literal type.
	// pub fn insert_literal_type<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	// 	&mut self,
	// 	context: &mut Context<M>,
	// 	vocabulary: &mut V,
	// 	generator: &mut impl Generator<V>,
	// 	Meta(lit, meta): Meta<crate::Literal, M>,
	// ) -> Result<Meta<Id, M>, Error<M>>
	// where
	// 	M: Clone + Merge,
	// {
	// 	use std::collections::btree_map::Entry;
	// 	match self.literal.entry(lit) {
	// 		Entry::Occupied(entry) => {
	// 			self.next_id.take();

	// 			let data = entry.get();
	// 			if !data.ty {
	// 				Self::generate_literal_type(
	// 					&data.id,
	// 					data.layout,
	// 					context,
	// 					Meta(entry.key(), &meta),
	// 				)?;
	// 			}

	// 			Ok(data.id.clone())
	// 		}
	// 		Entry::Vacant(entry) => {
	// 			let id = self
	// 				.next_id
	// 				.take()
	// 				.unwrap_or_else(|| Meta(generator.next(vocabulary), meta.clone()));

	// 			Self::generate_literal_type(&id, false, context, Meta(entry.key(), &meta))?;
	// 			entry.insert(LiteralData {
	// 				id: id.clone(),
	// 				ty: true,
	// 				layout: false,
	// 			});

	// 			Ok(id)
	// 		}
	// 	}
	// }

	// /// Inserts a new literal layout.
	// pub fn insert_literal_layout<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
	// 	&mut self,
	// 	context: &mut Context<M>,
	// 	vocabulary: &mut V,
	// 	generator: &mut impl Generator<V>,
	// 	Meta(lit, meta): Meta<crate::Literal, M>,
	// ) -> Result<Meta<Id, M>, Error<M>>
	// where
	// 	M: Clone + Merge,
	// {
	// 	use std::collections::btree_map::Entry;
	// 	match self.literal.entry(lit) {
	// 		Entry::Occupied(entry) => {
	// 			self.next_id.take();

	// 			let data = entry.get();
	// 			if !data.layout {
	// 				Self::generate_literal_layout(
	// 					&data.id,
	// 					data.ty,
	// 					context,
	// 					Meta(entry.key(), &meta),
	// 				)?;
	// 			}

	// 			Ok(data.id.clone())
	// 		}
	// 		Entry::Vacant(entry) => {
	// 			let id = self
	// 				.next_id
	// 				.take()
	// 				.unwrap_or_else(|| Meta(generator.next(vocabulary), meta.clone()));

	// 			Self::generate_literal_layout(&id, false, context, Meta(entry.key(), &meta))?;
	// 			entry.insert(LiteralData {
	// 				id: id.clone(),
	// 				ty: false,
	// 				layout: true,
	// 			});

	// 			Ok(id)
	// 		}
	// 	}
	// }
}
