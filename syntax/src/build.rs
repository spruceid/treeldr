use iref::{IriBuf, IriRef, IriRefBuf};
use locspan::{Loc, Location, Meta};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use treeldr::{reporting, vocab::*, Caused, Causes, Id, Name, Vocabulary, WithCauses};
use treeldr_build::{Context, ObjectToId};

mod intersection;

pub use intersection::*;

#[derive(Debug)]
pub enum Error<F> {
	Global(treeldr_build::Error<F>),
	Local(Loc<LocalError<F>, F>),
}

impl<F: Clone> reporting::DiagnoseWithVocabulary<F> for Error<F> {
	fn message(&self, vocab: &Vocabulary) -> String {
		match self {
			Self::Global(e) => e.message(vocab),
			Self::Local(e) => reporting::Diagnose::message(e),
		}
	}

	fn labels(&self, vocab: &Vocabulary) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		match self {
			Self::Global(e) => e.labels(vocab),
			Self::Local(e) => reporting::Diagnose::labels(e),
		}
	}
}

impl<F> From<treeldr_build::Error<F>> for Error<F> {
	fn from(e: treeldr_build::Error<F>) -> Self {
		Self::Global(e)
	}
}

impl<F> From<Loc<LocalError<F>, F>> for Error<F> {
	fn from(e: Loc<LocalError<F>, F>) -> Self {
		Self::Local(e)
	}
}

#[derive(Error, Debug)]
pub enum LocalError<F> {
	#[error("`{0}` is not a valid IRI")]
	InvalidExpandedCompactIri(String),
	#[error("prefix `{0}` is undefined")]
	UndefinedPrefix(String),
	#[error("prefix `{0}` is already defined")]
	AlreadyDefinedPrefix(String, Location<F>),
	#[error("cannot resolve the IRI reference without a base IRI")]
	NoBaseIri,
	#[error("should be `{expected}`")]
	BaseIriMismatch {
		expected: Box<IriBuf>,
		found: Box<IriBuf>,
		because: Location<F>,
	},
	#[error("type aliases are not supported")]
	TypeAlias(Id, Location<F>),
	#[error("only inline layouts can be assigned a name")]
	Renaming(Id, Location<F>),
	#[error("cannot define restricted field layout outside an intersection")]
	PropertyRestrictionOutsideIntersection,
	#[error("field not found")]
	FieldRestrictionNoMatches,
	#[error("unexpected field restriction")]
	UnexpectedFieldRestriction,
	#[error("field restrictions lead to anonymous layout")]
	AnonymousFieldLayoutIntersection(Vec<WithCauses<Id, F>>),
}

impl<F: Clone> reporting::DiagnoseWithCause<F> for LocalError<F> {
	fn message(&self, _cause: Option<&Location<F>>) -> String {
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

	fn labels(&self, cause: Option<&Location<F>>) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		let mut labels = Vec::new();

		if let Some(loc) = cause {
			labels.push(
				loc.clone()
					.into_primary_label()
					.with_message(self.to_string()),
			)
		}

		match self {
			Self::AlreadyDefinedPrefix(_, original_loc) => labels.push(
				original_loc
					.clone()
					.into_secondary_label()
					.with_message("original prefix defined here".to_string()),
			),
			Self::BaseIriMismatch { because, .. } => labels.push(
				because
					.clone()
					.into_secondary_label()
					.with_message("original base IRI defined here".to_string()),
			),
			Self::AnonymousFieldLayoutIntersection(layouts) => {
				for layout in layouts {
					if let Some(cause) = layout.causes().preferred() {
						labels.push(
							cause
								.clone()
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

impl<F: Clone + Ord> treeldr_build::Descriptions<F> for Descriptions {
	type Type = treeldr_build::ty::Description<F>;
	type Layout = LayoutDescription<F>;
}

/// Build context.
pub struct LocalContext<F> {
	/// Base IRI of th parsed document.
	base_iri: Option<IriBuf>,

	/// Bound prefixes.
	prefixes: HashMap<String, Loc<IriBuf, F>>,

	/// Current scope.
	scope: Option<Id>,

	next_id: Option<Loc<Id, F>>,

	label_id: HashMap<crate::Label, Loc<Id, F>>,

	/// Associates each literal type/value to a blank node label.
	literal: BTreeMap<Loc<crate::Literal, F>, LiteralData<F>>,

	/// Flag indicating if the (layout) definition is implicit.
	///
	/// If `true`, then the layout will be bound to itself.
	implicit_definition: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub struct LayoutRestrictedField<F> {
	field_prop: Option<Loc<Id, F>>,
	field_name: Option<Loc<Name, F>>,
	restriction: Loc<LayoutFieldRestriction, F>,
}

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone, PartialEq, Eq)]
pub enum LayoutFieldRestriction {
	Range(LayoutFieldRangeRestriction),
	Cardinality(LayoutFieldCardinalityRestriction),
}

#[derive(Clone)]
pub struct LiteralData<F> {
	id: Loc<Id, F>,
	ty: bool,
	layout: bool,
}

impl<F> LocalContext<F> {
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

	pub fn anonymous_id(
		&mut self,
		label: Option<crate::Label>,
		vocabulary: &mut Vocabulary,
		loc: Location<F>,
	) -> Loc<Id, F>
	where
		F: Clone + Ord,
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
							.unwrap_or_else(|| Loc(Id::Blank(vocabulary.new_blank_label()), loc));
						entry.insert(id.clone());
						id
					}
				}
			}
			None => self
				.next_id
				.take()
				.unwrap_or_else(|| Loc(Id::Blank(vocabulary.new_blank_label()), loc)),
		};

		self.next_id.take();
		id
	}
}

impl<F: Clone> LocalContext<F> {
	pub fn base_iri(
		&self,
		vocabulary: &mut Vocabulary,
		loc: Location<F>,
	) -> Result<IriBuf, Loc<LocalError<F>, F>> {
		match &self.scope {
			Some(Id::Iri(scope)) => {
				let mut iri = scope.iri(vocabulary).unwrap().to_owned();
				iri.path_mut().open();
				Ok(iri)
			}
			_ => self.base_iri.clone().ok_or(Loc(LocalError::NoBaseIri, loc)),
		}
	}

	pub fn set_base_iri(&mut self, base_iri: IriBuf) {
		self.base_iri = Some(base_iri)
	}

	pub fn declare_prefix(
		&mut self,
		prefix: String,
		iri: IriBuf,
		loc: Location<F>,
	) -> Result<(), Loc<LocalError<F>, F>> {
		use std::collections::hash_map::Entry;
		match self.prefixes.entry(prefix) {
			Entry::Occupied(entry) => Err(Loc(
				LocalError::AlreadyDefinedPrefix(
					entry.key().to_owned(),
					entry.get().location().clone(),
				),
				loc,
			)),
			Entry::Vacant(entry) => {
				entry.insert(Loc(iri, loc));
				Ok(())
			}
		}
	}

	pub fn expand_compact_iri(
		&self,
		prefix: &str,
		iri_ref: IriRef,
		loc: &Location<F>,
	) -> Result<IriBuf, Loc<LocalError<F>, F>> {
		match self.prefixes.get(prefix) {
			Some(iri) => match IriBuf::try_from(iri.as_str().to_string() + iri_ref.as_str()) {
				Ok(iri) => Ok(iri),
				Err((_, string)) => Err(Loc(
					LocalError::InvalidExpandedCompactIri(string),
					loc.clone(),
				)),
			},
			None => Err(Loc(
				LocalError::UndefinedPrefix(prefix.to_owned()),
				loc.clone(),
			)),
		}
	}

	pub fn generate_literal_type(
		Meta(id, _): &Loc<Id, F>,
		bind_to_layout: bool,
		context: &mut Context<F, Descriptions>,
		Meta(lit, loc): &Loc<crate::Literal, F>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		if id.is_blank() {
			// Define the type.
			context.declare_type(*id, Some(loc.clone()));
		}

		if bind_to_layout {
			context
				.get_mut(*id)
				.unwrap()
				.as_layout_mut()
				.unwrap()
				.set_type(*id, Some(loc.clone()))?;
		}

		let regexp = match lit {
			crate::Literal::String(s) => treeldr::ty::data::RegExp::from(s),
			crate::Literal::RegExp(regexp_string) => {
				match treeldr::ty::data::RegExp::parse(regexp_string) {
					Ok(regexp) => regexp,
					Err(e) => {
						return Err(treeldr_build::Error::new(
							treeldr_build::error::RegExpInvalid(regexp_string.clone(), e).into(),
							Some(loc.clone()),
						)
						.into())
					}
				}
			}
		};

		let ty = context.get_mut(*id).unwrap().as_type_mut().unwrap();

		let dt = ty.require_datatype_mut(Some(loc.clone()))?;
		dt.set_derivation_base(Id::Iri(Term::Xsd(Xsd::String)), Some(loc.clone()))?;
		let derived = dt.as_derived_mut().unwrap();
		derived.restrictions_mut().insert(
			treeldr_build::ty::data::Restriction::String(
				treeldr_build::ty::data::restriction::String::Pattern(regexp),
			),
			Some(loc.clone()),
		);

		Ok(())
	}

	pub fn generate_literal_layout(
		Meta(id, _): &Loc<Id, F>,
		bind_to_type: bool,
		context: &mut Context<F, Descriptions>,
		Meta(lit, loc): &Loc<crate::Literal, F>,
	) -> Result<(), Error<F>>
	where
		F: Clone + Ord,
	{
		if id.is_blank() {
			// Define the associated layout.
			context.declare_layout(*id, Some(loc.clone()));
		}

		if bind_to_type {
			context
				.get_mut(*id)
				.unwrap()
				.as_layout_mut()
				.unwrap()
				.set_type(*id, Some(loc.clone()))?;
		}

		let regexp = match lit {
			crate::Literal::String(s) => treeldr::ty::data::RegExp::from(s),
			crate::Literal::RegExp(regexp_string) => {
				match treeldr::ty::data::RegExp::parse(regexp_string) {
					Ok(regexp) => regexp,
					Err(e) => {
						return Err(treeldr_build::Error::new(
							treeldr_build::error::RegExpInvalid(regexp_string.clone(), e).into(),
							Some(loc.clone()),
						)
						.into())
					}
				}
			}
		};

		let mut restricted = treeldr_build::layout::RestrictedPrimitive::unrestricted(
			treeldr::layout::Primitive::String,
			Some(loc.clone()),
		);
		restricted.restrictions_mut().insert(
			treeldr_build::layout::primitive::Restriction::String(
				treeldr_build::layout::primitive::restriction::String::Pattern(regexp),
			),
			Some(loc.clone()),
		);

		context
			.get_mut(*id)
			.unwrap()
			.as_layout_mut()
			.unwrap()
			.set_primitive(restricted, Some(loc.clone()))?;

		Ok(())
	}

	/// Inserts a new literal type.
	pub fn insert_literal_type(
		&mut self,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
		lit: Loc<crate::Literal, F>,
	) -> Result<Loc<Id, F>, Error<F>>
	where
		F: Clone + Ord,
	{
		use std::collections::btree_map::Entry;
		match self.literal.entry(lit) {
			Entry::Occupied(entry) => {
				self.next_id.take();

				let data = entry.get();
				if !data.ty {
					Self::generate_literal_type(&data.id, data.layout, context, entry.key())?;
				}

				Ok(data.id.clone())
			}
			Entry::Vacant(entry) => {
				let loc = entry.key().location();
				let id = self
					.next_id
					.take()
					.unwrap_or_else(|| Loc(Id::Blank(vocabulary.new_blank_label()), loc.clone()));

				Self::generate_literal_type(&id, false, context, entry.key())?;
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
	pub fn insert_literal_layout(
		&mut self,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
		lit: Loc<crate::Literal, F>,
	) -> Result<Loc<Id, F>, Error<F>>
	where
		F: Clone + Ord,
	{
		use std::collections::btree_map::Entry;
		match self.literal.entry(lit) {
			Entry::Occupied(entry) => {
				self.next_id.take();

				let data = entry.get();
				if !data.layout {
					Self::generate_literal_layout(&data.id, data.ty, context, entry.key())?;
				}

				Ok(data.id.clone())
			}
			Entry::Vacant(entry) => {
				let loc = entry.key().location();
				let id = self
					.next_id
					.take()
					.unwrap_or_else(|| Loc(Id::Blank(vocabulary.new_blank_label()), loc.clone()));

				Self::generate_literal_layout(&id, false, context, entry.key())?;
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

impl<F: Clone + Ord> treeldr_build::Document<F, Descriptions> for crate::Document<F> {
	type LocalContext = LocalContext<F>;
	type Error = Error<F>;

	fn declare(
		&self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		let mut declared_base_iri = None;
		for Meta(base_iri, loc) in &self.bases {
			match declared_base_iri.take() {
				Some(Meta(declared_base_iri, d_loc)) => {
					if declared_base_iri != *base_iri {
						return Err(Loc(
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
					declared_base_iri = Some(Loc(base_iri.clone(), loc.clone()));
				}
			}
		}

		for import in &self.uses {
			import.declare(local_context, context, vocabulary)?
		}

		for ty in &self.types {
			ty.declare(local_context, context, vocabulary)?
		}

		for prop in &self.properties {
			prop.declare(local_context, context, vocabulary)?
		}

		for layout in &self.layouts {
			layout.declare(local_context, context, vocabulary)?
		}

		Ok(())
	}

	fn relate(
		self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		for ty in self.types {
			ty.build(local_context, context, vocabulary)?
		}

		for prop in self.properties {
			prop.build(local_context, context, vocabulary)?;
		}

		for layout in self.layouts {
			layout.build(local_context, context, vocabulary)?
		}

		Ok(())
	}
}

pub trait Declare<F: Clone + Ord> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>>;
}

pub trait Build<F: Clone + Ord> {
	type Target;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>>;
}

impl<F: Clone + Ord> Build<F> for Loc<crate::Documentation<F>, F> {
	type Target = (Option<String>, Option<String>);

	fn build(
		self,
		_local_context: &mut LocalContext<F>,
		_context: &mut Context<F, Descriptions>,
		_vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
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
					description_loc.span_mut().append(line_loc.span());
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
					label_loc.span_mut().append(line_loc.span());
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

impl<F: Clone + Ord> Build<F> for Loc<crate::Id, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		_context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
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

		Ok(Loc(Id::Iri(Term::from_iri(iri, vocabulary)), loc))
	}
}

impl<F: Clone + Ord> Declare<F> for Loc<crate::Use<F>, F> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		_context: &mut Context<F, Descriptions>,
		_vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		local_context
			.declare_prefix(
				self.prefix.value().as_str().to_string(),
				self.iri.value().clone(),
				self.location().clone(),
			)
			.map_err(Into::into)
	}
}

impl<F: Clone + Ord> Declare<F> for Loc<crate::TypeDefinition<F>, F> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		let Meta(id, _) = self.id.clone().build(local_context, context, vocabulary)?;
		context.declare_type(id, Some(self.location().clone()));

		if let Meta(crate::TypeDescription::Normal(properties), _) = &self.description {
			for prop in properties {
				local_context.scope = Some(id);
				prop.declare(local_context, context, vocabulary)?;
				local_context.scope = None
			}
		}

		Ok(())
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::TypeDefinition<F>, F> {
	type Target = ();

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		let implicit_layout = Loc(self.implicit_layout_definition(), self.location().clone());

		let Meta(def, _) = self;
		let Meta(id, id_loc) = def.id.build(local_context, context, vocabulary)?;

		match def.description {
			Meta(crate::TypeDescription::Normal(properties), _) => {
				for property in properties {
					local_context.scope = Some(id);
					let Meta(prop_id, prop_loc) =
						property.build(local_context, context, vocabulary)?;
					local_context.scope = None;

					let prop = context.get_mut(prop_id).unwrap().as_property_mut().unwrap();
					prop.set_domain(id, Some(prop_loc.clone()));
					let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
					ty.declare_property(prop_id, Some(prop_loc))?;
				}
			}
			Meta(crate::TypeDescription::Alias(expr), expr_loc) => {
				local_context.next_id = Some(Loc(id, id_loc));
				Loc(expr, expr_loc).build(local_context, context, vocabulary)?;
				local_context.next_id = None
			}
		}

		if let Some(doc) = def.doc {
			let (label, doc) = doc.build(local_context, context, vocabulary)?;
			let node = context.get_mut(id).unwrap();

			if let Some(label) = label {
				node.add_label(label);
			}

			if let Some(doc) = doc {
				node.documentation_mut().add(doc);
			}
		}

		local_context.implicit_definition = true;
		implicit_layout.declare(local_context, context, vocabulary)?;
		implicit_layout.build(local_context, context, vocabulary)?;
		local_context.implicit_definition = false;

		Ok(())
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::OuterTypeExpr<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(ty, loc) = self;

		match ty {
			crate::OuterTypeExpr::Inner(e) => Loc(e, loc).build(local_context, context, vocabulary),
			crate::OuterTypeExpr::Union(label, options) => {
				let Meta(id, _) = local_context.anonymous_id(Some(label), vocabulary, loc.clone());
				if id.is_blank() {
					context.declare_type(id, Some(loc.clone()));
				}

				let options_list = context.try_create_list_with::<Error<F>, _, _>(
					vocabulary,
					options,
					|ty_expr, context, vocabulary| {
						let Meta(id, loc) = ty_expr.build(local_context, context, vocabulary)?;
						Ok(Caused::new(id.into_term(), Some(loc)))
					},
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_union(options_list, Some(loc.clone()))?;

				Ok(Loc(id, loc))
			}
			crate::OuterTypeExpr::Intersection(label, types) => {
				let Meta(id, _) = local_context.anonymous_id(Some(label), vocabulary, loc.clone());
				if id.is_blank() {
					context.declare_type(id, Some(loc.clone()));
				}

				let types_list = context.try_create_list_with::<Error<F>, _, _>(
					vocabulary,
					types,
					|ty_expr, context, vocabulary| {
						let Meta(id, loc) = ty_expr.build(local_context, context, vocabulary)?;
						Ok(Caused::new(id.into_term(), Some(loc)))
					},
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_intersection(types_list, Some(loc.clone()))?;

				Ok(Loc(id, loc))
			}
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::NamedInnerTypeExpr<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		self.into_value()
			.expr
			.build(local_context, context, vocabulary)
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::InnerTypeExpr<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(ty, loc) = self;

		match ty {
			crate::InnerTypeExpr::Outer(outer) => outer.build(local_context, context, vocabulary),
			crate::InnerTypeExpr::Id(id) => {
				if let Some(Meta(id, id_loc)) = local_context.next_id.take() {
					return Err(Loc(LocalError::TypeAlias(id, id_loc), loc).into());
				}

				id.build(local_context, context, vocabulary)
			}
			crate::InnerTypeExpr::Reference(r) => r.build(local_context, context, vocabulary),
			crate::InnerTypeExpr::Literal(lit) => {
				local_context.insert_literal_type(context, vocabulary, Loc(lit, loc))
			}
			crate::InnerTypeExpr::PropertyRestriction(r) => {
				let Meta(id, loc) = local_context.anonymous_id(None, vocabulary, loc);
				if id.is_blank() {
					context.declare_type(id, Some(loc.clone()));
				}

				let prop_id = r.prop.build(local_context, context, vocabulary)?;
				let mut restrictions = treeldr_build::ty::Restriction::new(prop_id.into());

				let Meta(restriction, restriction_loc) = r.restriction;
				let restriction = match restriction {
					crate::TypePropertyRestriction::Range(r) => {
						let r = match r {
							crate::TypePropertyRangeRestriction::Any(id) => {
								let Meta(id, _) = id.build(local_context, context, vocabulary)?;
								treeldr_build::ty::RangeRestriction::Any(id)
							}
							crate::TypePropertyRangeRestriction::All(id) => {
								let Meta(id, _) = id.build(local_context, context, vocabulary)?;
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

				restrictions.add_restriction(restriction, Some(restriction_loc));

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_restriction(restrictions, Some(loc.clone()))?;

				Ok(Loc(id, loc))
			}
			crate::InnerTypeExpr::List(label, item) => {
				let Meta(id, _) = local_context.anonymous_id(Some(label), vocabulary, loc.clone());
				if id.is_blank() {
					context.declare_type(id, Some(loc.clone()));
				}

				let Meta(item_id, _) = item.build(local_context, context, vocabulary)?;

				// Restriction on the `rdf:first` property.
				let Meta(first_restriction_id, _) =
					local_context.anonymous_id(None, vocabulary, loc.clone());
				context.declare_type(first_restriction_id, Some(loc.clone()));
				let mut first_restriction = treeldr_build::ty::Restriction::new(WithCauses::new(
					Id::Iri(Term::Rdf(Rdf::First)),
					loc.clone(),
				));
				first_restriction.add_restriction(
					treeldr_build::ty::PropertyRestriction::Range(
						treeldr_build::ty::RangeRestriction::All(item_id),
					),
					Some(loc.clone()),
				);
				let first_restriction_ty = context
					.get_mut(first_restriction_id)
					.unwrap()
					.as_type_mut()
					.unwrap();
				first_restriction_ty.declare_restriction(first_restriction, Some(loc.clone()))?;

				// Restriction on the `rdf:rest` property.
				let Meta(rest_restriction_id, _) =
					local_context.anonymous_id(None, vocabulary, loc.clone());
				context.declare_type(rest_restriction_id, Some(loc.clone()));
				let mut rest_restriction = treeldr_build::ty::Restriction::new(WithCauses::new(
					Id::Iri(Term::Rdf(Rdf::Rest)),
					loc.clone(),
				));
				rest_restriction.add_restriction(
					treeldr_build::ty::PropertyRestriction::Range(
						treeldr_build::ty::RangeRestriction::All(id),
					),
					Some(loc.clone()),
				);
				let rest_restriction_ty = context
					.get_mut(rest_restriction_id)
					.unwrap()
					.as_type_mut()
					.unwrap();
				rest_restriction_ty.declare_restriction(rest_restriction, Some(loc.clone()))?;

				// Intersection list.
				let types_id = context.create_list(
					vocabulary,
					[
						Caused::new(Object::Iri(Term::Rdf(Rdf::List)), Some(loc.clone())),
						Caused::new(first_restriction_id.into_term(), Some(loc.clone())),
						Caused::new(rest_restriction_id.into_term(), Some(loc.clone())),
					],
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_intersection(types_id, Some(loc.clone()))?;

				Ok(Loc(id, loc))
			}
		}
	}
}

impl<F: Clone + Ord> Declare<F> for Loc<crate::PropertyDefinition<F>, F> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		let Meta(id, _) = self.id.clone().build(local_context, context, vocabulary)?;
		context.declare_property(id, Some(self.location().clone()));
		Ok(())
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::PropertyDefinition<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(def, loc) = self;
		let Meta(id, id_loc) = def.id.build(local_context, context, vocabulary)?;

		let doc = def
			.doc
			.map(|doc| doc.build(local_context, context, vocabulary))
			.transpose()?;

		let mut functional = true;
		let mut functional_loc = None;
		let mut required = false;
		let mut required_loc = None;

		let range = def
			.ty
			.map(|Meta(ty, _)| -> Result<_, Error<F>> {
				let scope = local_context.scope.take();
				let range = ty.expr.build(local_context, context, vocabulary)?;
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
					}
				}

				Ok(range)
			})
			.transpose()?;

		context.declare_property(id, Some(loc));
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
			prop.set_range(range, Some(range_loc))?;
		}

		if let Some(functional_loc) = functional_loc {
			prop.set_functional(functional, Some(functional_loc))?;
		}

		if let Some(required_loc) = required_loc {
			prop.set_required(required, Some(required_loc))?;
		}

		Ok(Loc(id, id_loc))
	}
}

impl<F: Clone + Ord> Declare<F> for Loc<crate::LayoutDefinition<F>, F> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		let Meta(id, _) = self.id.clone().build(local_context, context, vocabulary)?;
		context.declare_layout(id, Some(self.location().clone()));
		Ok(())
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::LayoutDefinition<F>, F> {
	type Target = ();

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<(), Error<F>> {
		let Meta(def, _) = self;
		let Meta(id, id_loc) = def.id.build(local_context, context, vocabulary)?;

		if let Some(doc) = def.doc {
			let (label, doc) = doc.build(local_context, context, vocabulary)?;
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
				let Meta(ty_id, ty_id_loc) = ty_id.build(local_context, context, vocabulary)?;
				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.set_type(ty_id, Some(ty_id_loc))?;
				Some(ty_id)
			}
			None => None,
		};

		match def.description {
			Meta(crate::LayoutDescription::Normal(fields), fields_loc) => {
				let fields_list = context.try_create_list_with::<Error<F>, _, _>(
					vocabulary,
					fields,
					|field, context, vocabulary| {
						local_context.scope = ty_id;
						let Meta(item, item_loc) =
							field.build(local_context, context, vocabulary)?;
						local_context.scope = None;
						Ok(Caused::new(item.into_term(), Some(item_loc)))
					},
				)?;

				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.set_fields(fields_list, Some(fields_loc))?;
			}
			Meta(crate::LayoutDescription::Alias(expr), expr_loc) => {
				local_context.next_id = Some(Loc(id, id_loc));
				Loc(expr, expr_loc).build(local_context, context, vocabulary)?;
				local_context.next_id = None;
			}
		}

		Ok(())
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::Alias, F> {
	type Target = Loc<treeldr::Name, F>;

	fn build(
		self,
		_local_context: &mut LocalContext<F>,
		_context: &mut Context<F, Descriptions>,
		_vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(name, loc) = self;
		match treeldr::Name::new(name.as_str()) {
			Ok(name) => Ok(Loc(name, loc)),
			Err(_) => Err(treeldr_build::Error::new(
				treeldr_build::error::NameInvalid(name.into_string()).into(),
				Some(loc),
			)
			.into()),
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::OuterLayoutExpr<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(ty, loc) = self;

		match ty {
			crate::OuterLayoutExpr::Inner(e) => {
				Loc(e, loc).build(local_context, context, vocabulary)
			}
			crate::OuterLayoutExpr::Union(label, options) => {
				let Meta(id, _) = local_context.anonymous_id(Some(label), vocabulary, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, Some(loc.clone()));
				}

				let variants = context.try_create_list_with::<Error<F>, _, _>(
					vocabulary,
					options,
					|layout_expr, context, vocabulary| {
						let loc = layout_expr.location().clone();
						let variant_id = Id::Blank(vocabulary.new_blank_label());

						let (layout_expr, variant_name) = if layout_expr.value().expr.is_namable() {
							let name = layout_expr.value().name.clone();
							(layout_expr, name)
						} else {
							let Meta(layout_expr, loc) = layout_expr;
							let (expr, name) = layout_expr.into_parts();
							(
								Loc(crate::NamedInnerLayoutExpr { expr, name: None }, loc),
								name,
							)
						};

						let Meta(layout, layout_loc) =
							layout_expr.build(local_context, context, vocabulary)?;

						let variant_name = variant_name
							.map(|name| name.build(local_context, context, vocabulary))
							.transpose()?;

						context.declare_layout_variant(variant_id, Some(loc.clone()));

						let variant = context
							.get_mut(variant_id)
							.unwrap()
							.as_layout_variant_mut()
							.unwrap();
						variant.set_layout(layout, Some(layout_loc))?;

						if let Some(Meta(name, name_loc)) = variant_name {
							variant.set_name(name, Some(name_loc))?
						}

						Ok(Caused::new(variant_id.into_term(), Some(loc)))
					},
				)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_enum(variants, Some(loc.clone()))?;
				if local_context.implicit_definition {
					layout.set_type(id, Some(loc.clone()))?;
				}

				Ok(Loc(id, loc))
			}
			crate::OuterLayoutExpr::Intersection(label, layouts) => {
				let Meta(id, _) = local_context.anonymous_id(Some(label), vocabulary, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, Some(loc.clone()));
				}

				let mut true_layouts = Vec::with_capacity(layouts.len());
				let mut restrictions = Vec::new();
				for Meta(layout_expr, loc) in layouts {
					match layout_expr.into_restriction() {
						Ok(restriction) => restrictions.push(Loc(restriction, loc).build(
							local_context,
							context,
							vocabulary,
						)?),
						Err(other) => true_layouts.push(Loc(other, loc)),
					}
				}

				let layouts_list = context.try_create_list_with::<Error<F>, _, _>(
					vocabulary,
					true_layouts,
					|layout_expr, context, vocabulary| {
						let Meta(id, loc) =
							layout_expr.build(local_context, context, vocabulary)?;
						Ok(Caused::new(id.into_term(), Some(loc)))
					},
				)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_description(
					LayoutDescription::Intersection(layouts_list, restrictions),
					Some(loc.clone()),
				)?;
				if local_context.implicit_definition {
					layout.set_type(id, Some(loc.clone()))?;
				}

				Ok(Loc(id, loc))
			}
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::LayoutRestrictedField<F>, F> {
	type Target = Loc<LayoutRestrictedField<F>, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(r, loc) = self;

		let field_name = match r.alias {
			Some(alias) => Some(alias.build(local_context, context, vocabulary)?),
			None => None,
		};

		Ok(Loc(
			LayoutRestrictedField {
				field_prop: Some(r.prop.build(local_context, context, vocabulary)?),
				field_name,
				restriction: r.restriction.build(local_context, context, vocabulary)?,
			},
			loc,
		))
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::LayoutFieldRestriction<F>, F> {
	type Target = Loc<LayoutFieldRestriction, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(r, loc) = self;

		let r = match r {
			crate::LayoutFieldRestriction::Range(r) => {
				LayoutFieldRestriction::Range(r.build(local_context, context, vocabulary)?)
			}
			crate::LayoutFieldRestriction::Cardinality(c) => LayoutFieldRestriction::Cardinality(c),
		};

		Ok(Loc(r, loc))
	}
}

impl<F: Clone + Ord> Build<F> for crate::LayoutFieldRangeRestriction<F> {
	type Target = LayoutFieldRangeRestriction;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		match self {
			Self::Any(layout_expr) => {
				let Meta(id, _) = layout_expr.build(local_context, context, vocabulary)?;
				Ok(LayoutFieldRangeRestriction::Any(id))
			}
			Self::All(layout_expr) => {
				let Meta(id, _) = layout_expr.build(local_context, context, vocabulary)?;
				Ok(LayoutFieldRangeRestriction::All(id))
			}
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::NamedInnerLayoutExpr<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(this, loc) = self;
		let is_namable = this.expr.is_namable();
		let Meta(id, expr_loc) = this.expr.build(local_context, context, vocabulary)?;

		if let Some(name) = this.name {
			let Meta(name, name_loc) = name.build(local_context, context, vocabulary)?;
			if is_namable {
				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.set_name(name, Some(name_loc))?;
			} else {
				return Err(Loc(LocalError::Renaming(id, expr_loc), loc).into());
			}
		}

		Ok(Loc(id, loc))
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::InnerLayoutExpr<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(expr, loc) = self;

		match expr {
			crate::InnerLayoutExpr::Outer(outer) => outer.build(local_context, context, vocabulary),
			crate::InnerLayoutExpr::Id(id) => {
				let alias_id = local_context.next_id.take();

				let id = id.build(local_context, context, vocabulary)?;

				match alias_id {
					Some(Meta(alias_id, alias_id_loc)) => {
						if alias_id.is_blank() {
							context.declare_layout(alias_id, Some(alias_id_loc));
						}

						let layout = context.get_mut(alias_id).unwrap().as_layout_mut().unwrap();
						layout.set_alias(*id, Some(loc.clone()))?;

						Ok(Loc(alias_id, loc))
					}
					None => Ok(id),
				}
			}
			crate::InnerLayoutExpr::Primitive(p) => {
				let Meta(id, _) = local_context.anonymous_id(None, vocabulary, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, Some(loc.clone()));
				}

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_primitive(
					treeldr_build::layout::primitive::Restricted::unrestricted(
						p,
						Some(loc.clone()),
					),
					Some(loc.clone()),
				)?;

				Ok(Loc(id, loc))
			}
			crate::InnerLayoutExpr::Reference(ty_expr) => {
				let id = local_context.next_id.take();

				let Meta(deref_ty, deref_loc) =
					ty_expr.build(local_context, context, vocabulary)?;

				let id = match id {
					Some(Meta(id, _)) => {
						if id.is_blank() {
							context.declare_layout(id, Some(loc.clone()));
						}

						let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
						layout.set_type(deref_ty, Some(deref_loc))?;
						let id_layout = Id::Iri(Term::TreeLdr(TreeLdr::Primitive(
							treeldr::layout::Primitive::Iri,
						)));
						layout.set_reference(id_layout, Some(loc.clone()))?;
						id
					}
					None => context.standard_reference(
						vocabulary,
						deref_ty,
						Some(loc.clone()),
						Some(deref_loc),
					)?,
				};

				Ok(Loc(id, loc))
			}
			crate::InnerLayoutExpr::Literal(lit) => {
				local_context.insert_literal_layout(context, vocabulary, Loc(lit, loc))
			}
			crate::InnerLayoutExpr::FieldRestriction(_) => {
				Err(Loc(LocalError::PropertyRestrictionOutsideIntersection, loc).into())
			}
			crate::InnerLayoutExpr::Array(label, item) => {
				let Meta(id, _) = local_context.anonymous_id(Some(label), vocabulary, loc.clone());
				if id.is_blank() {
					context.declare_layout(id, Some(loc.clone()));
				}

				let Meta(item_id, _) = item.build(local_context, context, vocabulary)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				let semantics = if local_context.implicit_definition {
					layout.set_type(id, Some(loc.clone()))?;
					Some(treeldr_build::layout::array::Semantics::rdf_list(Some(
						loc.clone(),
					)))
				} else {
					None
				};

				layout.set_array(item_id, semantics, Some(loc.clone()))?;

				Ok(Loc(id, loc))
			}
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::FieldDefinition<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
		vocabulary: &mut Vocabulary,
	) -> Result<Self::Target, Error<F>> {
		let Meta(def, loc) = self;

		let id = Id::Blank(vocabulary.new_blank_label());

		let Meta(prop_id, prop_id_loc) = def.id.build(local_context, context, vocabulary)?;

		let Meta(name, name_loc) = def
			.alias
			.unwrap_or_else(|| match prop_id {
				Id::Iri(id) => {
					let iri = id.iri(vocabulary).unwrap();

					let id = match iri.fragment() {
						Some(fragment) => fragment.to_string(),
						None => iri
							.path()
							.file_name()
							.expect("invalid property IRI")
							.to_owned(),
					};

					Loc(crate::Alias(id), prop_id_loc.clone())
				}
				_ => panic!("invalid property IRI"),
			})
			.build(local_context, context, vocabulary)?;

		let mut required = false;
		let mut multiple = false;
		let mut multiple_loc = None;

		let layout = match def.layout {
			Some(Meta(layout, _)) => {
				let scope = local_context.scope.take();
				let layout_id = layout.expr.build(local_context, context, vocabulary)?;
				local_context.scope = scope;

				for Meta(ann, ann_loc) in layout.annotations {
					match ann {
						crate::Annotation::Multiple => {
							multiple = true;
							multiple_loc = Some(ann_loc)
						}
						crate::Annotation::Required => required = true,
					}
				}

				let layout_id = if required {
					if multiple {
						// Wrap inside non-empty set.
						let container_id = Id::Blank(vocabulary.new_blank_label());
						context.declare_layout(container_id, multiple_loc.clone());
						let container_layout = context
							.get_mut(container_id)
							.unwrap()
							.as_layout_mut()
							.unwrap();
						let Meta(item_layout_id, item_layout_loc) = layout_id;
						container_layout.set_set(item_layout_id, multiple_loc)?;
						Loc(container_id, item_layout_loc)
					} else {
						// Wrap inside non-empty set.
						let container_id = Id::Blank(vocabulary.new_blank_label());
						context.declare_layout(container_id, None);
						let container_layout = context
							.get_mut(container_id)
							.unwrap()
							.as_layout_mut()
							.unwrap();
						let Meta(item_layout_id, item_layout_loc) = layout_id;
						container_layout.set_required(item_layout_id, None)?;
						Loc(container_id, item_layout_loc)
					}
				} else if multiple {
					// Wrap inside set.
					let container_id = Id::Blank(vocabulary.new_blank_label());
					context.declare_layout(container_id, multiple_loc.clone());
					let container_layout = context
						.get_mut(container_id)
						.unwrap()
						.as_layout_mut()
						.unwrap();
					let Meta(item_layout_id, item_layout_loc) = layout_id;
					container_layout.set_set(item_layout_id, multiple_loc)?;
					Loc(container_id, item_layout_loc)
				} else {
					// Wrap inside option.
					let container_id = Id::Blank(vocabulary.new_blank_label());
					context.declare_layout(container_id, None);
					let container_layout = context
						.get_mut(container_id)
						.unwrap()
						.as_layout_mut()
						.unwrap();
					let Meta(item_layout_id, item_layout_loc) = layout_id;
					container_layout.set_option(item_layout_id, None)?;
					Loc(container_id, item_layout_loc)
				};

				Some(layout_id)
			}
			None => None,
		};

		let doc = def
			.doc
			.map(|doc| doc.build(local_context, context, vocabulary))
			.transpose()?;

		context.declare_layout_field(id, Some(loc.clone()));
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
		field.set_property(prop_id, Some(prop_id_loc))?;
		field.set_name(name, Some(name_loc))?;

		if let Some(Meta(layout, layout_loc)) = layout {
			field.set_layout(layout, Some(layout_loc))?;
		}

		Ok(Loc(id, loc))
	}
}

#[derive(Clone)]
pub enum LayoutDescription<F> {
	/// Standard layout description.
	Standard(treeldr_build::layout::Description<F>),

	Intersection(Id, Vec<Loc<LayoutRestrictedField<F>, F>>),
}

impl<F: Clone + Ord> LayoutDescription<F> {
	pub fn simplify(
		self,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
		causes: &Causes<F>,
	) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
		match self {
			Self::Standard(desc) => Ok(desc),
			Self::Intersection(id, restricted_fields) => {
				let layout_list = source.require_list(id, causes.preferred().cloned())?;
				let mut layouts = Vec::new();
				for obj in layout_list.iter(source) {
					let obj = obj?;
					layouts.push(WithCauses::new(
						obj.as_id(obj.causes().preferred())?,
						obj.causes().clone(),
					))
				}

				let mut result = IntersectedLayout::try_from_iter(layouts, source, causes.clone())?;
				result = result.apply_restrictions(restricted_fields)?;
				result.into_standard_description(source, target, vocabulary)
			}
		}
	}
}

impl<F: Clone + Ord> treeldr_build::Simplify<F> for Descriptions {
	type Error = Error<F>;
	type TryMap = TrySimplify;
}

#[derive(Default)]
pub struct TrySimplify;

impl<F: Clone + Ord>
	treeldr_build::TryMap<F, Error<F>, Descriptions, treeldr_build::StandardDescriptions>
	for TrySimplify
{
	fn ty(
		&self,
		a: treeldr_build::ty::Description<F>,
		_causes: &Causes<F>,
		_source: &Context<F, Descriptions>,
		_target: &mut Context<F>,
		_vocabulary: &mut Vocabulary,
	) -> Result<treeldr_build::ty::Description<F>, Error<F>> {
		Ok(a)
	}

	fn layout(
		&self,
		a: LayoutDescription<F>,
		causes: &Causes<F>,
		source: &Context<F, Descriptions>,
		target: &mut Context<F>,
		vocabulary: &mut Vocabulary,
	) -> Result<treeldr_build::layout::Description<F>, Error<F>> {
		a.simplify(source, target, vocabulary, causes)
	}
}

impl<F> From<treeldr_build::layout::Description<F>> for LayoutDescription<F> {
	fn from(d: treeldr_build::layout::Description<F>) -> Self {
		Self::Standard(d)
	}
}

impl<F: Clone + Ord> treeldr_build::layout::PseudoDescription<F> for LayoutDescription<F> {
	fn as_standard(&self) -> Option<&treeldr_build::layout::Description<F>> {
		match self {
			Self::Standard(s) => Some(s),
			_ => None,
		}
	}

	fn as_standard_mut(&mut self) -> Option<&mut treeldr_build::layout::Description<F>> {
		match self {
			Self::Standard(s) => Some(s),
			_ => None,
		}
	}

	fn try_unify(
		self,
		other: Self,
		id: Id,
		causes: &Causes<F>,
		other_causes: &Causes<F>,
	) -> Result<Self, treeldr_build::Error<F>> {
		match (self, other) {
			(Self::Standard(a), Self::Standard(b)) => {
				Ok(Self::Standard(a.try_unify(b, id, causes, other_causes)?))
			}
			(Self::Intersection(a, a_restrictions), Self::Intersection(b, b_restrictions)) => {
				if a == b && a_restrictions == b_restrictions {
					Ok(Self::Intersection(a, a_restrictions))
				} else {
					Err(treeldr_build::Error::new(
						treeldr_build::error::LayoutMismatchDescription {
							id,
							because: causes.preferred().cloned(),
						}
						.into(),
						other_causes.preferred().cloned(),
					))
				}
			}
			_ => Err(treeldr_build::Error::new(
				treeldr_build::error::LayoutMismatchDescription {
					id,
					because: causes.preferred().cloned(),
				}
				.into(),
				other_causes.preferred().cloned(),
			)),
		}
	}
}
