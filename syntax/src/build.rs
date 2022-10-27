use iref::{IriBuf, IriRef, IriRefBuf};
use locspan::{BorrowStripped, MapLocErr, MaybeLocated, Meta, Span};
use locspan_derive::{StrippedEq, StrippedPartialEq};
use rdf_types::{Generator, Vocabulary, VocabularyMut};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use treeldr::{metadata::Merge, reporting, vocab::*, Id, Name};
use treeldr_build::{error, Context, ObjectToId};

mod intersection;

pub use intersection::*;

#[derive(Debug)]
pub enum Error<M> {
	Global(treeldr_build::Error<M>),
	Local(Meta<LocalError<M>, M>),
}

impl<M: Clone + MaybeLocated<Span = Span>> reporting::DiagnoseWithVocabulary<M> for Error<M>
where
	M::File: Clone,
{
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		match self {
			Self::Global(e) => e.message(vocab),
			Self::Local(e) => reporting::Diagnose::message(e),
		}
	}

	fn labels(
		&self,
		vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
	) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		match self {
			Self::Global(e) => e.labels(vocab),
			Self::Local(e) => reporting::Diagnose::labels(e),
		}
	}
}

impl<M> From<treeldr_build::Error<M>> for Error<M> {
	fn from(e: treeldr_build::Error<M>) -> Self {
		Self::Global(e)
	}
}

impl<M> From<Meta<LocalError<M>, M>> for Error<M> {
	fn from(e: Meta<LocalError<M>, M>) -> Self {
		Self::Local(e)
	}
}

#[derive(Error, Debug)]
pub enum LocalError<M> {
	#[error("`{0}` is not a valid IRI")]
	InvalidExpandedCompactIri(String),
	#[error("prefix `{0}` is undefined")]
	UndefinedPrefix(String),
	#[error("prefix `{0}` is already defined")]
	AlreadyDefinedPrefix(String, M),
	#[error("cannot resolve the IRI reference without a base IRI")]
	NoBaseIri,
	#[error("should be `{expected}`")]
	BaseIriMismatch {
		expected: Box<IriBuf>,
		found: Box<IriBuf>,
		because: M,
	},
	#[error("type aliases are not supported")]
	TypeAlias(Id, M),
	#[error("only inline layouts can be assigned a name")]
	Renaming(Id, M),
	#[error("cannot define restricted field layout outside an intersection")]
	PropertyRestrictionOutsideIntersection,
	#[error("field not found")]
	FieldRestrictionNoMatches,
	#[error("unexpected field restriction")]
	UnexpectedFieldRestriction,
	#[error("field restrictions lead to anonymous layout")]
	AnonymousFieldLayoutIntersection(Vec<Meta<Id, M>>),
}

impl<M: Clone + MaybeLocated<Span = Span>> reporting::DiagnoseWithMetadata<M> for LocalError<M>
where
	M::File: Clone,
{
	fn message(&self, _cause: &M) -> String {
		match self {
			Self::InvalidExpandedCompactIri(_) => "invalid expanded compact IRI".to_string(),
			Self::UndefinedPrefix(_) => "undefined prefix".to_string(),
			Self::AlreadyDefinedPrefix(_, _) => "already defined prefix".to_string(),
			Self::NoBaseIri => "no base IRI".to_string(),
			Self::BaseIriMismatch { .. } => "base IRI mismatch".to_string(),
			Self::TypeAlias(_, _) => "type aliases are not supported".to_string(),
			Self::Renaming(_, _) => "invalid layout renaming".to_string(),
			Self::PropertyRestrictionOutsideIntersection => {
				"cannot define restricted field layout outside an intersection".to_string()
			}
			Self::FieldRestrictionNoMatches => "no matches for field restriction".to_string(),
			Self::UnexpectedFieldRestriction => {
				"field restrictions can only be applied on structure layouts".to_string()
			}
			Self::AnonymousFieldLayoutIntersection(_) => "unexpected anonymous layout".to_string(),
		}
	}

	fn labels(&self, cause: &M) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		let mut labels = Vec::new();

		if let Some(loc) = cause.optional_location() {
			labels.push(
				loc.clone()
					.into_primary_label()
					.with_message(self.to_string()),
			)
		}

		match self {
			Self::AlreadyDefinedPrefix(_, original_meta) => {
				if let Some(loc) = original_meta.optional_location() {
					labels.push(
						loc.clone()
							.into_secondary_label()
							.with_message("original prefix defined here".to_string()),
					)
				}
			}
			Self::BaseIriMismatch { because, .. } => {
				if let Some(loc) = because.optional_location() {
					labels.push(
						loc.clone()
							.into_secondary_label()
							.with_message("original base IRI defined here".to_string()),
					)
				}
			}
			Self::AnonymousFieldLayoutIntersection(layouts) => {
				for layout in layouts {
					if let Some(loc) = layout.metadata().optional_location() {
						labels.push(
							loc.clone()
								.into_secondary_label()
								.with_message("part of the intersection".to_string()),
						)
					}
				}
			}
			_ => (),
		}

		labels
	}
}

pub struct Descriptions;

impl<M: Clone + Merge> treeldr_build::Descriptions<M> for Descriptions {
	type Type = treeldr_build::ty::Description<M>;
	type Layout = LayoutDescription<M>;
}

/// Build context.
pub struct LocalContext<M> {
	/// Base IRI of th parsed document.
	base_iri: Option<IriBuf>,

	/// Bound prefixes.
	prefixes: HashMap<String, Meta<IriBuf, M>>,

	/// Current scope.
	scope: Option<Id>,

	next_id: Option<Meta<Id, M>>,

	label_id: HashMap<crate::Label, Meta<Id, M>>,

	/// Associates each literal type/value to a blank node label.
	literal: BTreeMap<crate::Literal, LiteralData<M>>,

	/// Flag indicating if the (layout) definition is implicit.
	///
	/// If `true`, then the layout will be bound to itself.
	implicit_definition: bool,
}

#[derive(Clone, PartialEq, Eq, StrippedPartialEq, StrippedEq)]
#[stripped_ignore(M)]
pub struct LayoutRestrictedField<M> {
	#[stripped_option_deref]
	field_prop: Option<Meta<Id, M>>,

	#[stripped_option_deref]
	field_name: Option<Meta<Name, M>>,

	restriction: Meta<LayoutFieldRestriction, M>,
}

#[derive(Clone, PartialEq, Eq, StrippedPartialEq, StrippedEq)]
pub enum LayoutFieldRangeRestriction {
	Any(Id),
	All(Id),
}

impl LayoutFieldRangeRestriction {
	pub fn layout(&self) -> Id {
		match self {
			Self::Any(id) => *id,
			Self::All(id) => *id,
		}
	}
}

pub type LayoutFieldCardinalityRestriction = crate::LayoutFieldCardinalityRestriction;

#[derive(Clone, PartialEq, Eq, StrippedPartialEq, StrippedEq)]
pub enum LayoutFieldRestriction {
	Range(LayoutFieldRangeRestriction),
	Cardinality(LayoutFieldCardinalityRestriction),
}

#[derive(Clone)]
pub struct LiteralData<M> {
	id: Meta<Id, M>,
	ty: bool,
	layout: bool,
}

impl<M> LocalContext<M> {
	pub fn new(base_iri: Option<IriBuf>) -> Self {
		Self {
			base_iri,
			prefixes: HashMap::new(),
			scope: None,
			next_id: None,
			label_id: HashMap::new(),
			literal: BTreeMap::new(),
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
		let id = match label {
			Some(label) => {
				use std::collections::hash_map::Entry;
				match self.label_id.entry(label) {
					Entry::Occupied(entry) => entry.get().clone(),
					Entry::Vacant(entry) => {
						let id = self
							.next_id
							.take()
							.unwrap_or_else(|| Meta(generator.next(vocabulary), loc));
						entry.insert(id.clone());
						id
					}
				}
			}
			None => self
				.next_id
				.take()
				.unwrap_or_else(|| Meta(generator.next(vocabulary), loc)),
		};

		self.next_id.take();
		id
	}
}

impl<M: Clone> LocalContext<M> {
	pub fn base_iri(
		&self,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
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

	pub fn generate_literal_type(
		Meta(id, _): &Meta<Id, M>,
		bind_to_layout: bool,
		context: &mut Context<M, Descriptions>,
		Meta(lit, loc): Meta<&crate::Literal, &M>,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		if id.is_blank() {
			// Define the type.
			context.declare_type(*id, loc.clone());
		}

		if bind_to_layout {
			context
				.get_mut(*id)
				.unwrap()
				.as_layout_mut()
				.unwrap()
				.set_type(*id, loc.clone())?;
		}

		let regexp = match lit {
			crate::Literal::String(s) => treeldr::ty::data::RegExp::from(s),
			crate::Literal::RegExp(regexp_string) => {
				match treeldr::ty::data::RegExp::parse(regexp_string) {
					Ok(regexp) => regexp,
					Err(e) => {
						return Err(treeldr_build::Error::new(
							treeldr_build::error::RegExpInvalid(regexp_string.clone(), e).into(),
							loc.clone(),
						)
						.into())
					}
				}
			}
		};

		let ty = context.get_mut(*id).unwrap().as_type_mut().unwrap();

		let dt = ty.require_datatype_mut(loc)?;
		dt.set_derivation_base(Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::String))), loc.clone())?;
		let derived = dt.as_derived_mut().unwrap();
		derived.restrictions_mut().insert(
			treeldr_build::ty::data::Restriction::String(
				treeldr_build::ty::data::restriction::String::Pattern(regexp),
			),
			loc.clone(),
		);

		Ok(())
	}

	pub fn generate_literal_layout(
		Meta(id, _): &Meta<Id, M>,
		bind_to_type: bool,
		context: &mut Context<M, Descriptions>,
		Meta(lit, loc): Meta<&crate::Literal, &M>,
	) -> Result<(), Error<M>>
	where
		M: Clone + Merge,
	{
		if id.is_blank() {
			// Define the associated layout.
			context.declare_layout(*id, loc.clone());
		}

		if bind_to_type {
			context
				.get_mut(*id)
				.unwrap()
				.as_layout_mut()
				.unwrap()
				.set_type(*id, loc.clone())?;
		}

		let regexp = match lit {
			crate::Literal::String(s) => treeldr::ty::data::RegExp::from(s),
			crate::Literal::RegExp(regexp_string) => {
				match treeldr::ty::data::RegExp::parse(regexp_string) {
					Ok(regexp) => regexp,
					Err(e) => {
						return Err(treeldr_build::Error::new(
							treeldr_build::error::RegExpInvalid(regexp_string.clone(), e).into(),
							loc.clone(),
						)
						.into())
					}
				}
			}
		};

		let layout = context.get_mut(*id).unwrap().as_layout_mut().unwrap();

		layout.set_primitive(treeldr::layout::Primitive::String, loc.clone())?;
		layout.restrictions_mut().primitive.insert(
			treeldr_build::layout::primitive::Restriction::String(
				treeldr_build::layout::primitive::restriction::String::Pattern(regexp),
			),
			loc.clone(),
		);

		Ok(())
	}

	/// Inserts a new literal type.
	pub fn insert_literal_type<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		Meta(lit, meta): Meta<crate::Literal, M>,
	) -> Result<Meta<Id, M>, Error<M>>
	where
		M: Clone + Merge,
	{
		use std::collections::btree_map::Entry;
		match self.literal.entry(lit) {
			Entry::Occupied(entry) => {
				self.next_id.take();

				let data = entry.get();
				if !data.ty {
					Self::generate_literal_type(
						&data.id,
						data.layout,
						context,
						Meta(entry.key(), &meta),
					)?;
				}

				Ok(data.id.clone())
			}
			Entry::Vacant(entry) => {
				let id = self
					.next_id
					.take()
					.unwrap_or_else(|| Meta(generator.next(vocabulary), meta.clone()));

				Self::generate_literal_type(&id, false, context, Meta(entry.key(), &meta))?;
				entry.insert(LiteralData {
					id: id.clone(),
					ty: true,
					layout: false,
				});

				Ok(id)
			}
		}
	}

	/// Inserts a new literal layout.
	pub fn insert_literal_layout<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&mut self,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		Meta(lit, meta): Meta<crate::Literal, M>,
	) -> Result<Meta<Id, M>, Error<M>>
	where
		M: Clone + Merge,
	{
		use std::collections::btree_map::Entry;
		match self.literal.entry(lit) {
			Entry::Occupied(entry) => {
				self.next_id.take();

				let data = entry.get();
				if !data.layout {
					Self::generate_literal_layout(
						&data.id,
						data.ty,
						context,
						Meta(entry.key(), &meta),
					)?;
				}

				Ok(data.id.clone())
			}
			Entry::Vacant(entry) => {
				let id = self
					.next_id
					.take()
					.unwrap_or_else(|| Meta(generator.next(vocabulary), meta.clone()));

				Self::generate_literal_layout(&id, false, context, Meta(entry.key(), &meta))?;
				entry.insert(LiteralData {
					id: id.clone(),
					ty: false,
					layout: true,
				});

				Ok(id)
			}
		}
	}
}

impl<M: Clone + Merge> treeldr_build::Document<M, Descriptions> for crate::Document<M> {
	type LocalContext = LocalContext<M>;
	type Error = Error<M>;

	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M, Descriptions>,
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

	fn relate<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<M, Descriptions>,
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
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>>;
}

pub trait Build<M: Clone + Merge> {
	type Target;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>>;
}

impl<M: Clone + Merge> Build<M> for Meta<crate::Documentation<M>, M> {
	type Target = (Option<String>, Option<String>);

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		_local_context: &mut LocalContext<M>,
		_context: &mut Context<M, Descriptions>,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(doc, loc) = self;
		let mut label = String::new();
		let mut label_loc = loc.clone();

		let mut description = String::new();
		let mut description_loc = loc;

		let mut separated = false;

		for Meta(line, line_loc) in doc.items {
			let line = line.trim();

			if separated {
				if description.is_empty() {
					description_loc = line_loc;
				} else {
					description_loc.merge_with(line_loc);
				}

				if !description.is_empty() {
					description.push('\n');
				}

				description.push_str(line);
			} else if line.trim().is_empty() {
				separated = true
			} else {
				if label.is_empty() {
					label_loc = line_loc;
				} else {
					label_loc.merge_with(line_loc);
				}

				label.push_str(line);
			}
		}

		let label = if label.is_empty() { None } else { Some(label) };

		let description = if description.is_empty() {
			None
		} else {
			Some(description)
		};

		Ok((label, description))
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::Id, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		_context: &mut Context<M, Descriptions>,
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
		_context: &mut Context<M, Descriptions>,
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

impl<M: Clone + Merge> Declare<M> for Meta<crate::TypeDefinition<M>, M> {
	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let Meta(id, _) = self
			.id
			.clone()
			.build(local_context, context, vocabulary, generator)?;
		context.declare_type(id, self.metadata().clone());

		if let Meta(crate::TypeDescription::Normal(properties), _) = &self.description {
			for prop in properties {
				local_context.scope = Some(id);
				prop.declare(local_context, context, vocabulary, generator)?;
				local_context.scope = None
			}
		}

		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::TypeDefinition<M>, M> {
	type Target = ();

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let implicit_layout = Meta(self.implicit_layout_definition(), self.metadata().clone());

		let Meta(def, _) = self;
		let Meta(id, id_loc) = def
			.id
			.build(local_context, context, vocabulary, generator)?;

		match def.description {
			Meta(crate::TypeDescription::Normal(properties), _) => {
				for property in properties {
					local_context.scope = Some(id);
					let Meta(prop_id, prop_loc) =
						property.build(local_context, context, vocabulary, generator)?;
					local_context.scope = None;

					let prop = context.get_mut(prop_id).unwrap().as_property_mut().unwrap();
					prop.set_domain(id, prop_loc.clone());
					let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
					ty.declare_property(prop_id, prop_loc)?;
				}
			}
			Meta(crate::TypeDescription::Alias(expr), expr_loc) => {
				local_context.next_id = Some(Meta(id, id_loc));
				Meta(expr, expr_loc).build(local_context, context, vocabulary, generator)?;
				local_context.next_id = None
			}
		}

		if let Some(doc) = def.doc {
			let (label, doc) = doc.build(local_context, context, vocabulary, generator)?;
			let node = context.get_mut(id).unwrap();

			if let Some(label) = label {
				node.add_label(label);
			}

			if let Some(doc) = doc {
				node.documentation_mut().add(doc);
			}
		}

		local_context.implicit_definition = true;
		implicit_layout.declare(local_context, context, vocabulary, generator)?;
		implicit_layout.build(local_context, context, vocabulary, generator)?;
		local_context.implicit_definition = false;

		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::OuterTypeExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(ty, loc) = self;

		match ty {
			crate::OuterTypeExpr::Inner(e) => {
				Meta(e, loc).build(local_context, context, vocabulary, generator)
			}
			crate::OuterTypeExpr::Union(label, options) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_type(id, loc.clone());
				}

				let options_list = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					options,
					|ty_expr, context, vocabulary, generator| {
						let Meta(id, loc) =
							ty_expr.build(local_context, context, vocabulary, generator)?;
						Ok(Meta(id.into_term(), loc))
					},
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_union(options_list, loc.clone())?;

				Ok(Meta(id, loc))
			}
			crate::OuterTypeExpr::Intersection(label, types) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_type(id, loc.clone());
				}

				let types_list = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					types,
					|ty_expr, context, vocabulary, generator| {
						let Meta(id, loc) =
							ty_expr.build(local_context, context, vocabulary, generator)?;
						Ok(Meta(id.into_term(), loc))
					},
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_intersection(types_list, loc.clone())?;

				Ok(Meta(id, loc))
			}
		}
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::NamedInnerTypeExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		self.into_value()
			.expr
			.build(local_context, context, vocabulary, generator)
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::InnerTypeExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(ty, loc) = self;

		match ty {
			crate::InnerTypeExpr::Outer(outer) => {
				outer.build(local_context, context, vocabulary, generator)
			}
			crate::InnerTypeExpr::Id(id) => {
				if let Some(Meta(id, id_loc)) = local_context.next_id.take() {
					return Err(Meta(LocalError::TypeAlias(id, id_loc), loc).into());
				}

				id.build(local_context, context, vocabulary, generator)
			}
			crate::InnerTypeExpr::Reference(r) => {
				r.build(local_context, context, vocabulary, generator)
			}
			crate::InnerTypeExpr::Literal(lit) => {
				local_context.insert_literal_type(context, vocabulary, generator, Meta(lit, loc))
			}
			crate::InnerTypeExpr::PropertyRestriction(r) => {
				let Meta(id, loc) = local_context.anonymous_id(None, vocabulary, generator, loc);
				if id.is_blank() {
					context.declare_type(id, loc.clone());
				}

				let prop_id = r
					.prop
					.build(local_context, context, vocabulary, generator)?;
				let mut restrictions = treeldr_build::ty::Restriction::new(prop_id);

				let Meta(restriction, restriction_loc) = r.restriction;
				let restriction = match restriction {
					crate::TypePropertyRestriction::Range(r) => {
						let r = match r {
							crate::TypePropertyRangeRestriction::Any(id) => {
								let Meta(id, _) =
									id.build(local_context, context, vocabulary, generator)?;
								treeldr_build::ty::RangeRestriction::Any(id)
							}
							crate::TypePropertyRangeRestriction::All(id) => {
								let Meta(id, _) =
									id.build(local_context, context, vocabulary, generator)?;
								treeldr_build::ty::RangeRestriction::All(id)
							}
						};

						treeldr_build::ty::PropertyRestriction::Range(r)
					}
					crate::TypePropertyRestriction::Cardinality(c) => {
						let c = match c {
							crate::TypePropertyCardinalityRestriction::AtLeast(min) => {
								treeldr_build::ty::CardinalityRestriction::AtLeast(min)
							}
							crate::TypePropertyCardinalityRestriction::AtMost(max) => {
								treeldr_build::ty::CardinalityRestriction::AtMost(max)
							}
							crate::TypePropertyCardinalityRestriction::Exactly(n) => {
								treeldr_build::ty::CardinalityRestriction::Exactly(n)
							}
						};

						treeldr_build::ty::PropertyRestriction::Cardinality(c)
					}
				};

				restrictions.add_restriction(restriction, restriction_loc);

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_restriction(restrictions, loc.clone())?;

				Ok(Meta(id, loc))
			}
			crate::InnerTypeExpr::List(label, item) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_type(id, loc.clone());
				}

				let Meta(item_id, _) = item.build(local_context, context, vocabulary, generator)?;

				// Restriction on the `rdf:first` property.
				let Meta(first_restriction_id, _) =
					local_context.anonymous_id(None, vocabulary, generator, loc.clone());
				context.declare_type(first_restriction_id, loc.clone());
				let mut first_restriction = treeldr_build::ty::Restriction::new(Meta::new(
					Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
					loc.clone(),
				));
				first_restriction.add_restriction(
					treeldr_build::ty::PropertyRestriction::Range(
						treeldr_build::ty::RangeRestriction::All(item_id),
					),
					loc.clone(),
				);
				let first_restriction_ty = context
					.get_mut(first_restriction_id)
					.unwrap()
					.as_type_mut()
					.unwrap();
				first_restriction_ty.declare_restriction(first_restriction, loc.clone())?;

				// Restriction on the `rdf:rest` property.
				let Meta(rest_restriction_id, _) =
					local_context.anonymous_id(None, vocabulary, generator, loc.clone());
				context.declare_type(rest_restriction_id, loc.clone());
				let mut rest_restriction = treeldr_build::ty::Restriction::new(Meta::new(
					Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
					loc.clone(),
				));
				rest_restriction.add_restriction(
					treeldr_build::ty::PropertyRestriction::Range(
						treeldr_build::ty::RangeRestriction::All(id),
					),
					loc.clone(),
				);
				let rest_restriction_ty = context
					.get_mut(rest_restriction_id)
					.unwrap()
					.as_type_mut()
					.unwrap();
				rest_restriction_ty.declare_restriction(rest_restriction, loc.clone())?;

				// Intersection list.
				let types_id = context.create_list(
					vocabulary,
					generator,
					[
						Meta(
							Object::Iri(IriIndex::Iri(Term::Rdf(Rdf::List))),
							loc.clone(),
						),
						Meta(first_restriction_id.into_term(), loc.clone()),
						Meta(rest_restriction_id.into_term(), loc.clone()),
					],
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_intersection(types_id, loc.clone())?;

				Ok(Meta(id, loc))
			}
		}
	}
}

impl<M: Clone + Merge> Declare<M> for Meta<crate::PropertyDefinition<M>, M> {
	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let Meta(id, _) = self
			.id
			.clone()
			.build(local_context, context, vocabulary, generator)?;
		context.declare_property(id, self.metadata().clone());
		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::PropertyDefinition<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(def, loc) = self;
		let Meta(id, id_loc) = def
			.id
			.build(local_context, context, vocabulary, generator)?;

		let doc = def
			.doc
			.map(|doc| doc.build(local_context, context, vocabulary, generator))
			.transpose()?;

		let mut functional = true;
		let mut functional_loc = None;
		let mut required = false;
		let mut required_loc = None;

		let range = def
			.ty
			.map(|Meta(ty, _)| -> Result<_, Error<M>> {
				let scope = local_context.scope.take();
				let range = ty
					.expr
					.build(local_context, context, vocabulary, generator)?;
				local_context.scope = scope;

				for Meta(ann, ann_loc) in ty.annotations {
					match ann {
						crate::Annotation::Multiple => {
							functional = false;
							functional_loc = Some(ann_loc);
						}
						crate::Annotation::Required => {
							required = true;
							required_loc = Some(ann_loc);
						}
						crate::Annotation::Single => (),
					}
				}

				Ok(range)
			})
			.transpose()?;

		context.declare_property(id, loc);
		let node = context.get_mut(id).unwrap();
		if let Some((label, doc)) = doc {
			if let Some(label) = label {
				node.add_label(label);
			}

			if let Some(doc) = doc {
				node.documentation_mut().add(doc);
			}
		}

		let prop = node.as_property_mut().unwrap();
		if let Some(Meta(range, range_loc)) = range {
			prop.set_range(range, range_loc)?;
		}

		if let Some(functional_loc) = functional_loc {
			prop.set_functional(functional, functional_loc)?;
		}

		if let Some(required_loc) = required_loc {
			prop.set_required(required, required_loc)?;
		}

		Ok(Meta(id, id_loc))
	}
}

impl<M: Clone + Merge> Declare<M> for Meta<crate::LayoutDefinition<M>, M> {
	fn declare<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let Meta(id, _) = self
			.id
			.clone()
			.build(local_context, context, vocabulary, generator)?;
		context.declare_layout(id, self.metadata().clone());
		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::LayoutDefinition<M>, M> {
	type Target = ();

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<(), Error<M>> {
		let Meta(def, _) = self;
		let Meta(id, id_loc) = def
			.id
			.build(local_context, context, vocabulary, generator)?;

		if let Some(doc) = def.doc {
			let (label, doc) = doc.build(local_context, context, vocabulary, generator)?;
			let node = context.get_mut(id).unwrap();

			if let Some(label) = label {
				node.add_label(label);
			}

			if let Some(doc) = doc {
				node.documentation_mut().add(doc);
			}
		}

		let ty_id = match def.ty_id {
			Some(ty_id) => {
				let Meta(ty_id, ty_id_loc) =
					ty_id.build(local_context, context, vocabulary, generator)?;
				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.set_type(ty_id, ty_id_loc)?;
				Some(ty_id)
			}
			None => None,
		};

		match def.description {
			Meta(crate::LayoutDescription::Normal(fields), fields_loc) => {
				let fields_list = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					fields,
					|field, context, vocabulary, generator| {
						local_context.scope = ty_id;
						let Meta(item, item_loc) =
							field.build(local_context, context, vocabulary, generator)?;
						local_context.scope = None;
						Ok(Meta(item.into_term(), item_loc))
					},
				)?;

				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.set_fields(fields_list, fields_loc)?;
			}
			Meta(crate::LayoutDescription::Alias(expr), expr_loc) => {
				local_context.next_id = Some(Meta(id, id_loc));
				Meta(expr, expr_loc).build(local_context, context, vocabulary, generator)?;
				local_context.next_id = None;
			}
		}

		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::Alias, M> {
	type Target = Meta<treeldr::Name, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		_local_context: &mut LocalContext<M>,
		_context: &mut Context<M, Descriptions>,
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

impl<M: Clone + Merge> Build<M> for Meta<crate::OuterLayoutExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(ty, loc) = self;

		match ty {
			crate::OuterLayoutExpr::Inner(e) => {
				Meta(e, loc).build(local_context, context, vocabulary, generator)
			}
			crate::OuterLayoutExpr::Union(label, options) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let variants = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					options,
					|layout_expr, context, vocabulary, generator| {
						let loc = layout_expr.metadata().clone();
						let variant_id = generator.next(vocabulary);

						let (layout_expr, variant_name) = if layout_expr.value().expr.is_namable() {
							let name = layout_expr.value().name.clone();
							(layout_expr, name)
						} else {
							let Meta(layout_expr, loc) = layout_expr;
							let (expr, name) = layout_expr.into_parts();
							(
								Meta(crate::NamedInnerLayoutExpr { expr, name: None }, loc),
								name,
							)
						};

						let Meta(layout, layout_loc) =
							layout_expr.build(local_context, context, vocabulary, generator)?;

						let variant_name = variant_name
							.map(|name| name.build(local_context, context, vocabulary, generator))
							.transpose()?;

						context.declare_layout_variant(variant_id, loc.clone());

						let variant = context
							.get_mut(variant_id)
							.unwrap()
							.as_layout_variant_mut()
							.unwrap();
						variant.set_layout(layout, layout_loc)?;

						if let Some(Meta(name, name_loc)) = variant_name {
							variant.set_name(name, name_loc)?
						}

						Ok(Meta(variant_id.into_term(), loc))
					},
				)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_enum(variants, loc.clone())?;
				if local_context.implicit_definition {
					layout.set_type(id, loc.clone())?;
				}

				Ok(Meta(id, loc))
			}
			crate::OuterLayoutExpr::Intersection(label, layouts) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let mut true_layouts = Vec::with_capacity(layouts.len());
				let mut restrictions = Vec::new();
				for Meta(layout_expr, loc) in layouts {
					match layout_expr.into_restriction() {
						Ok(restriction) => restrictions.push(Meta(restriction, loc).build(
							local_context,
							context,
							vocabulary,
							generator,
						)?),
						Err(other) => true_layouts.push(Meta(other, loc)),
					}
				}

				let layouts_list = context.try_create_list_with::<Error<M>, _, _, _, _>(
					vocabulary,
					generator,
					true_layouts,
					|layout_expr, context, vocabulary, generator| {
						let Meta(id, loc) =
							layout_expr.build(local_context, context, vocabulary, generator)?;
						Ok(Meta(id.into_term(), loc))
					},
				)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_description(
					LayoutDescription::Intersection(layouts_list, restrictions),
					loc.clone(),
				)?;
				if local_context.implicit_definition {
					layout.set_type(id, loc.clone())?;
				}

				Ok(Meta(id, loc))
			}
		}
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::LayoutRestrictedField<M>, M> {
	type Target = Meta<LayoutRestrictedField<M>, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(r, loc) = self;

		let field_name = match r.alias {
			Some(alias) => Some(alias.build(local_context, context, vocabulary, generator)?),
			None => None,
		};

		Ok(Meta(
			LayoutRestrictedField {
				field_prop: Some(
					r.prop
						.build(local_context, context, vocabulary, generator)?,
				),
				field_name,
				restriction: r
					.restriction
					.build(local_context, context, vocabulary, generator)?,
			},
			loc,
		))
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::LayoutFieldRestriction<M>, M> {
	type Target = Meta<LayoutFieldRestriction, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(r, loc) = self;

		let r = match r {
			crate::LayoutFieldRestriction::Range(r) => LayoutFieldRestriction::Range(r.build(
				local_context,
				context,
				vocabulary,
				generator,
			)?),
			crate::LayoutFieldRestriction::Cardinality(c) => LayoutFieldRestriction::Cardinality(c),
		};

		Ok(Meta(r, loc))
	}
}

impl<M: Clone + Merge> Build<M> for crate::LayoutFieldRangeRestriction<M> {
	type Target = LayoutFieldRangeRestriction;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		match self {
			Self::Any(layout_expr) => {
				let Meta(id, _) =
					layout_expr.build(local_context, context, vocabulary, generator)?;
				Ok(LayoutFieldRangeRestriction::Any(id))
			}
			Self::All(layout_expr) => {
				let Meta(id, _) =
					layout_expr.build(local_context, context, vocabulary, generator)?;
				Ok(LayoutFieldRangeRestriction::All(id))
			}
		}
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::NamedInnerLayoutExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(this, loc) = self;
		let is_namable = this.expr.is_namable();
		let Meta(id, expr_loc) = this
			.expr
			.build(local_context, context, vocabulary, generator)?;

		if let Some(name) = this.name {
			let Meta(name, name_loc) = name.build(local_context, context, vocabulary, generator)?;
			if is_namable {
				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.set_name(name, name_loc)?;
			} else {
				return Err(Meta(LocalError::Renaming(id, expr_loc), loc).into());
			}
		}

		Ok(Meta(id, loc))
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::InnerLayoutExpr<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(expr, loc) = self;

		match expr {
			crate::InnerLayoutExpr::Outer(outer) => {
				outer.build(local_context, context, vocabulary, generator)
			}
			crate::InnerLayoutExpr::Id(id) => {
				let alias_id = local_context.next_id.take();

				let id = id.build(local_context, context, vocabulary, generator)?;

				match alias_id {
					Some(Meta(alias_id, alias_id_loc)) => {
						if alias_id.is_blank() {
							context.declare_layout(alias_id, alias_id_loc);
						}

						let layout = context.get_mut(alias_id).unwrap().as_layout_mut().unwrap();
						layout.set_alias(*id, loc.clone())?;

						Ok(Meta(alias_id, loc))
					}
					None => Ok(id),
				}
			}
			crate::InnerLayoutExpr::Primitive(p) => {
				let Meta(id, _) =
					local_context.anonymous_id(None, vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_primitive(p, loc.clone())?;

				Ok(Meta(id, loc))
			}
			crate::InnerLayoutExpr::Reference(ty_expr) => {
				let id = local_context.next_id.take();

				let Meta(deref_ty, deref_loc) =
					ty_expr.build(local_context, context, vocabulary, generator)?;

				let id = match id {
					Some(Meta(id, _)) => {
						if id.is_blank() {
							context.declare_layout(id, loc.clone());
						}

						let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
						layout.set_type(deref_ty, deref_loc)?;
						let id_layout = Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Primitive(
							treeldr::layout::Primitive::Iri,
						))));
						layout.set_reference(id_layout, loc.clone())?;
						id
					}
					None => context.standard_reference(
						vocabulary,
						generator,
						deref_ty,
						loc.clone(),
						deref_loc,
					)?,
				};

				Ok(Meta(id, loc))
			}
			crate::InnerLayoutExpr::Literal(lit) => {
				local_context.insert_literal_layout(context, vocabulary, generator, Meta(lit, loc))
			}
			crate::InnerLayoutExpr::FieldRestriction(_) => {
				Err(Meta(LocalError::PropertyRestrictionOutsideIntersection, loc).into())
			}
			crate::InnerLayoutExpr::Array(label, item) => {
				let Meta(id, _) =
					local_context.anonymous_id(Some(label), vocabulary, generator, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, loc.clone());
				}

				let Meta(item_id, _) = item.build(local_context, context, vocabulary, generator)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				let semantics = if local_context.implicit_definition {
					layout.set_type(id, loc.clone())?;
					Some(treeldr_build::layout::array::Semantics::rdf_list(
						loc.clone(),
					))
				} else {
					None
				};

				layout.set_array(item_id, semantics, loc.clone())?;

				Ok(Meta(id, loc))
			}
		}
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::FieldDefinition<M>, M> {
	type Target = Meta<Id, M>;

	fn build<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M, Descriptions>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(def, loc) = self;

		let id = generator.next(vocabulary);

		let Meta(prop_id, prop_id_loc) =
			def.id
				.build(local_context, context, vocabulary, generator)?;

		let Meta(name, name_loc) = def
			.alias
			.unwrap_or_else(|| match prop_id {
				Id::Iri(id) => {
					let iri = vocabulary.iri(&id).unwrap();

					let id = match iri.fragment() {
						Some(fragment) => fragment.to_string(),
						None => iri
							.path()
							.file_name()
							.expect("invalid property IRI")
							.to_owned(),
					};

					Meta(crate::Alias(id), prop_id_loc.clone())
				}
				_ => panic!("invalid property IRI"),
			})
			.build(local_context, context, vocabulary, generator)?;

		let mut required_loc = None;
		let mut multiple_loc = None;
		let mut single_loc = None;

		let layout = match def.layout {
			Some(Meta(layout, _)) => {
				let scope = local_context.scope.take();
				let layout_id = layout
					.expr
					.build(local_context, context, vocabulary, generator)?;
				local_context.scope = scope;

				for Meta(ann, ann_loc) in layout.annotations {
					match ann {
						crate::Annotation::Multiple => multiple_loc = Some(ann_loc),
						crate::Annotation::Required => required_loc = Some(ann_loc),
						crate::Annotation::Single => single_loc = Some(ann_loc),
					}
				}

				let layout_id = match required_loc {
					Some(required_loc) => {
						match multiple_loc {
							Some(multiple_loc) => {
								use treeldr::layout::container::restriction::Conflict;
								// Wrap inside non-empty set.
								let container_id = generator.next(vocabulary);
								context.declare_layout(container_id, multiple_loc.clone());
								let container_layout = context
									.get_mut(container_id)
									.unwrap()
									.as_layout_mut()
									.unwrap();
								let Meta(item_layout_id, item_layout_loc) = layout_id;

								match single_loc {
									Some(single_loc) => {
										container_layout.set_one_or_many(
											item_layout_id,
											multiple_loc.merged_with(single_loc),
										)?;
									}
									None => {
										container_layout.set_set(item_layout_id, multiple_loc)?;
									}
								}

								container_layout
									.restrictions_mut()
									.container
									.cardinal_mut()
									.insert_min(Meta(1, required_loc))
									.map_loc_err(|e| {
										error::Description::LayoutContainerRestrictionConflict(
											Conflict::Cardinal(e),
										)
									})?;
								Meta(container_id, item_layout_loc)
							}
							None => {
								// Wrap inside non-empty set.
								let container_id = generator.next(vocabulary);
								context.declare_layout(container_id, required_loc.clone());
								let container_layout = context
									.get_mut(container_id)
									.unwrap()
									.as_layout_mut()
									.unwrap();
								let Meta(item_layout_id, item_layout_loc) = layout_id;
								container_layout.set_required(item_layout_id, required_loc)?;
								Meta(container_id, item_layout_loc)
							}
						}
					}
					None => {
						match multiple_loc {
							Some(multiple_loc) => {
								// Wrap inside set.
								let container_id = generator.next(vocabulary);
								context.declare_layout(container_id, multiple_loc.clone());
								let container_layout = context
									.get_mut(container_id)
									.unwrap()
									.as_layout_mut()
									.unwrap();
								let Meta(item_layout_id, item_layout_loc) = layout_id;

								match single_loc {
									Some(single_loc) => {
										container_layout.set_one_or_many(
											item_layout_id,
											multiple_loc.merged_with(single_loc),
										)?;
									}
									None => {
										container_layout.set_set(item_layout_id, multiple_loc)?;
									}
								}

								Meta(container_id, item_layout_loc)
							}
							None => {
								// Wrap inside option.
								let container_id = generator.next(vocabulary);
								let Meta(item_layout_id, item_layout_loc) = layout_id;
								context.declare_layout(container_id, item_layout_loc.clone());
								let container_layout = context
									.get_mut(container_id)
									.unwrap()
									.as_layout_mut()
									.unwrap();
								container_layout
									.set_option(item_layout_id, item_layout_loc.clone())?;
								Meta(container_id, item_layout_loc)
							}
						}
					}
				};

				Some(layout_id)
			}
			None => None,
		};

		let doc = def
			.doc
			.map(|doc| doc.build(local_context, context, vocabulary, generator))
			.transpose()?;

		context.declare_layout_field(id, loc.clone());
		let node = context.get_mut(id).unwrap();
		if let Some((label, doc)) = doc {
			if let Some(label) = label {
				node.add_label(label);
			}

			if let Some(doc) = doc {
				node.documentation_mut().add(doc);
			}
		}

		let field = node.as_layout_field_mut().unwrap();
		field.set_property(prop_id, prop_id_loc)?;
		field.set_name(name, name_loc)?;

		if let Some(Meta(layout, layout_loc)) = layout {
			field.set_layout(layout, layout_loc)?;
		}

		Ok(Meta(id, loc))
	}
}

#[derive(Clone)]
pub enum LayoutDescription<M> {
	/// Standard layout description.
	Standard(treeldr_build::layout::Description<M>),

	Intersection(Id, Vec<Meta<LayoutRestrictedField<M>, M>>),
}

impl<M: Clone> LayoutDescription<M> {
	pub fn simplify<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		self,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
		causes: &M,
	) -> Result<treeldr_build::layout::Description<M>, Error<M>>
	where
		M: Merge,
	{
		match self {
			Self::Standard(desc) => Ok(desc),
			Self::Intersection(id, restricted_fields) => {
				let layout_list = source.require_list(id, causes)?;
				let mut layouts = Vec::new();
				for obj in layout_list.iter(source) {
					let obj = obj?;
					layouts.push(Meta::new(
						obj.as_id(obj.metadata())?,
						obj.metadata().clone(),
					))
				}

				let mut result = IntersectedLayout::try_from_iter(layouts, source, causes.clone())?;
				result = result.apply_restrictions(restricted_fields)?;
				result.into_standard_description(source, target, vocabulary, generator)
			}
		}
	}
}

impl<M: Clone + Merge> treeldr_build::Simplify<M> for Descriptions {
	type Error = Error<M>;
	type TryMap = TrySimplify;
}

#[derive(Default)]
pub struct TrySimplify;

impl<M: Clone + Merge>
	treeldr_build::TryMap<M, Error<M>, Descriptions, treeldr_build::StandardDescriptions>
	for TrySimplify
{
	fn ty<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		a: treeldr_build::ty::Description<M>,
		_causes: &M,
		_source: &Context<M, Descriptions>,
		_target: &mut Context<M>,
		_vocabulary: &mut V,
		_generator: &mut impl Generator<V>,
	) -> Result<treeldr_build::ty::Description<M>, Error<M>> {
		Ok(a)
	}

	fn layout<V: VocabularyMut<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		a: LayoutDescription<M>,
		causes: &M,
		source: &Context<M, Descriptions>,
		target: &mut Context<M>,
		vocabulary: &mut V,
		generator: &mut impl Generator<V>,
	) -> Result<treeldr_build::layout::Description<M>, Error<M>> {
		a.simplify(source, target, vocabulary, generator, causes)
	}
}

impl<M> From<treeldr_build::layout::Description<M>> for LayoutDescription<M> {
	fn from(d: treeldr_build::layout::Description<M>) -> Self {
		Self::Standard(d)
	}
}

impl<M: Clone + Merge> treeldr_build::layout::PseudoDescription<M> for LayoutDescription<M> {
	fn as_standard(&self) -> Option<&treeldr_build::layout::Description<M>> {
		match self {
			Self::Standard(s) => Some(s),
			_ => None,
		}
	}

	fn as_standard_mut(&mut self) -> Option<&mut treeldr_build::layout::Description<M>> {
		match self {
			Self::Standard(s) => Some(s),
			_ => None,
		}
	}

	fn try_unify(
		self,
		other: Self,
		id: Id,
		causes: M,
		other_causes: M,
	) -> Result<Meta<Self, M>, treeldr_build::Error<M>> {
		match (self, other) {
			(Self::Standard(a), Self::Standard(b)) => {
				let Meta(desc, meta) = a.try_unify(b, id, causes, other_causes)?;
				Ok(Meta(Self::Standard(desc), meta))
			}
			(Self::Intersection(a, a_restrictions), Self::Intersection(b, b_restrictions)) => {
				if a == b && a_restrictions.stripped() == b_restrictions.stripped() {
					Ok(Meta(
						Self::Intersection(a, a_restrictions),
						causes.merged_with(other_causes),
					))
				} else {
					Err(treeldr_build::Error::new(
						treeldr_build::error::LayoutMismatchDescription {
							id,
							because: causes,
						}
						.into(),
						other_causes,
					))
				}
			}
			_ => Err(treeldr_build::Error::new(
				treeldr_build::error::LayoutMismatchDescription {
					id,
					because: causes,
				}
				.into(),
				other_causes,
			)),
		}
	}
}
