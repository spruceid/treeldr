use iref::{IriBuf, IriRef, IriRefBuf};
use locspan::{Loc, Location};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use treeldr::{reporting, vocab::*, Caused, Causes, Id, MaybeSet, Vocabulary};
use treeldr_build::{context, Context, Item, SubLayout, Dependencies, utils::TryCollect};

#[derive(Debug)]
pub enum Error<F> {
	Global(treeldr_build::Error<F>),
	Local(Loc<LocalError<F>, F>),
}

impl<'c, 't, F: Clone> reporting::DiagnoseWithVocabulary<F> for Error<F> {
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

#[derive(Debug)]
pub enum LocalError<F> {
	InvalidExpandedCompactIri(String),
	UndefinedPrefix(String),
	AlreadyDefinedPrefix(String, Location<F>),
	NoBaseIri,
	BaseIriMismatch {
		expected: Box<IriBuf>,
		found: Box<IriBuf>,
		because: Location<F>,
	},
	TypeAlias(Id, Location<F>),
	LayoutAlias(Id, Location<F>),
}

impl<'c, 't, F: Clone> reporting::DiagnoseWithCause<F> for LocalError<F> {
	fn message(&self, _cause: Option<&Location<F>>) -> String {
		match self {
			Self::InvalidExpandedCompactIri(_) => "invalid expanded compact IRI".to_string(),
			Self::UndefinedPrefix(_) => "undefined prefix".to_string(),
			Self::AlreadyDefinedPrefix(_, _) => "already defined prefix".to_string(),
			Self::NoBaseIri => "no base IRI".to_string(),
			Self::BaseIriMismatch { .. } => "base IRI mismatch".to_string(),
			Self::TypeAlias(_, _) => "type aliases are not supported".to_string(),
			Self::LayoutAlias(_, _) => "layout aliases are not supported".to_string(),
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
			_ => (),
		}

		labels
	}
}

impl<F> fmt::Display for LocalError<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidExpandedCompactIri(expanded) => {
				write!(f, "`{}` is not a valid IRI", expanded)
			}
			Self::UndefinedPrefix(prefix) => write!(f, "prefix `{}` is undefined", prefix),
			Self::AlreadyDefinedPrefix(prefix, _) => {
				write!(f, "prefix `{}` is already defined", prefix)
			}
			Self::NoBaseIri => "cannot resolve the IRI reference without a base IRI".fmt(f),
			Self::BaseIriMismatch { expected, .. } => write!(f, "should be `{}`", expected),
			Self::TypeAlias(_, _) => write!(f, "type aliases are not supported"),
			Self::LayoutAlias(_, _) => write!(f, "layout aliases are not supported"),
		}
	}
}

pub struct Descriptions;

impl<F: Clone + Ord> treeldr_build::Descriptions<F> for Descriptions {
	type Error = Error<F>;

	type Type = treeldr_build::ty::Description<F>;
	type Layout = LayoutDescription;
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
		Loc(id, _): &Loc<Id, F>,
		bind_to_layout: bool,
		context: &mut Context<F, Descriptions>,
		Loc(_, loc): &Loc<crate::Literal, F>,
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

		Ok(())
	}

	pub fn generate_literal_layout(
		Loc(id, _): &Loc<Id, F>,
		bind_to_type: bool,
		context: &mut Context<F, Descriptions>,
		Loc(lit, loc): &Loc<crate::Literal, F>,
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
			crate::Literal::String(s) => treeldr::layout::literal::RegExp::from(s),
			crate::Literal::RegExp(regexp_string) => {
				match treeldr::layout::literal::RegExp::parse(regexp_string) {
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

		context
			.get_mut(*id)
			.unwrap()
			.as_layout_mut()
			.unwrap()
			.set_literal(regexp, Some(loc.clone()))?;
		Ok(())
	}

	/// Inserts a new literal type.
	pub fn insert_literal_type(
		&mut self,
		context: &mut Context<F, Descriptions>,
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
				let id = self.next_id.take().unwrap_or_else(|| {
					Loc(
						Id::Blank(context.vocabulary_mut().new_blank_label()),
						loc.clone(),
					)
				});

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
				let id = self.next_id.take().unwrap_or_else(|| {
					Loc(
						Id::Blank(context.vocabulary_mut().new_blank_label()),
						loc.clone(),
					)
				});

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
	) -> Result<(), Error<F>> {
		let mut declared_base_iri = None;
		for Loc(base_iri, loc) in &self.bases {
			match declared_base_iri.take() {
				Some(Loc(declared_base_iri, d_loc)) => {
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
			import.declare(local_context, context)?
		}

		for ty in &self.types {
			ty.declare(local_context, context)?
		}

		for layout in &self.layouts {
			layout.declare(local_context, context)?
		}

		Ok(())
	}

	fn relate(
		self,
		local_context: &mut Self::LocalContext,
		context: &mut Context<F, Descriptions>,
	) -> Result<(), Error<F>> {
		for ty in self.types {
			ty.build(local_context, context)?
		}

		for layout in self.layouts {
			layout.build(local_context, context)?
		}

		Ok(())
	}
}

pub trait Declare<F: Clone + Ord> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
	) -> Result<(), Error<F>>;
}

pub trait Build<F: Clone + Ord> {
	type Target;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
	) -> Result<Self::Target, Error<F>>;
}

impl<F: Clone + Ord> Build<F> for Loc<crate::Documentation<F>, F> {
	type Target = (Option<String>, Option<String>);

	fn build(
		self,
		_local_context: &mut LocalContext<F>,
		_context: &mut Context<F, Descriptions>,
	) -> Result<Self::Target, Error<F>> {
		let Loc(doc, loc) = self;
		let mut label = String::new();
		let mut label_loc = loc.clone();

		let mut description = String::new();
		let mut description_loc = loc;

		let mut separated = false;

		for Loc(line, line_loc) in doc.items {
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
		context: &mut Context<F, Descriptions>,
	) -> Result<Self::Target, Error<F>> {
		let Loc(id, loc) = self;
		let iri = match id {
			crate::Id::Name(name) => {
				let mut iri_ref = IriRefBuf::from_string(name).unwrap();
				iri_ref.resolve(
					local_context
						.base_iri(context.vocabulary_mut(), loc.clone())?
						.as_iri(),
				);
				iri_ref.try_into().unwrap()
			}
			crate::Id::IriRef(iri_ref) => iri_ref.resolved(
				local_context
					.base_iri(context.vocabulary_mut(), loc.clone())?
					.as_iri(),
			),
			crate::Id::Compact(prefix, iri_ref) => {
				local_context.expand_compact_iri(&prefix, iri_ref.as_iri_ref(), &loc)?
			}
		};

		Ok(Loc(
			Id::Iri(Term::from_iri(iri, context.vocabulary_mut())),
			loc,
		))
	}
}

impl<F: Clone + Ord> Declare<F> for Loc<crate::Use<F>, F> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		_context: &mut Context<F, Descriptions>,
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
	) -> Result<(), Error<F>> {
		let Loc(id, _) = self.id.clone().build(local_context, context)?;
		context.declare_type(id, Some(self.location().clone()));

		if let Loc(crate::TypeDescription::Normal(properties), _) = &self.description {
			for prop in properties {
				local_context.scope = Some(id);
				prop.declare(local_context, context)?;
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
	) -> Result<(), Error<F>> {
		let implicit_layout = Loc(self.implicit_layout_definition(), self.location().clone());

		let Loc(def, _) = self;
		let Loc(id, id_loc) = def.id.build(local_context, context)?;

		match def.description {
			Loc(crate::TypeDescription::Normal(properties), _) => {
				for property in properties {
					local_context.scope = Some(id);
					let Loc(prop_id, prop_loc) = property.build(local_context, context)?;
					local_context.scope = None;

					let prop = context.get_mut(prop_id).unwrap().as_property_mut().unwrap();
					prop.set_domain(id, Some(prop_loc.clone()));
					let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
					ty.declare_property(prop_id, Some(prop_loc))?;
				}
			}
			Loc(crate::TypeDescription::Alias(expr), expr_loc) => {
				local_context.next_id = Some(Loc(id, id_loc));
				Loc(expr, expr_loc).build(local_context, context)?;
				local_context.next_id = None
			}
		}

		if let Some(doc) = def.doc {
			let (label, doc) = doc.build(local_context, context)?;
			let node = context.get_mut(id).unwrap();

			if let Some(label) = label {
				node.add_label(label);
			}

			if let Some(doc) = doc {
				node.documentation_mut().add(doc);
			}
		}

		local_context.implicit_definition = true;
		implicit_layout.declare(local_context, context)?;
		implicit_layout.build(local_context, context)?;
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
	) -> Result<Self::Target, Error<F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::OuterTypeExpr::Inner(e) => Loc(e, loc).build(local_context, context),
			crate::OuterTypeExpr::Union(label, options) => {
				let Loc(id, _) =
					local_context.anonymous_id(Some(label), context.vocabulary_mut(), loc.clone());
				if id.is_blank() {
					context.declare_type(id, Some(loc.clone()));
				}

				let options_list = context.try_create_list_with::<Error<F>, _, _>(
					options,
					|ty_expr, context| {
						let Loc(id, loc) = ty_expr.build(local_context, context)?;
						Ok(Caused::new(id.into_term(), Some(loc)))
					},
				)?;

				let ty = context.get_mut(id).unwrap().as_type_mut().unwrap();
				ty.declare_union(options_list, Some(loc.clone()))?;

				Ok(Loc(id, loc))
			}
			crate::OuterTypeExpr::Intersection(label, types) => {
				let Loc(id, _) =
					local_context.anonymous_id(Some(label), context.vocabulary_mut(), loc.clone());
				if id.is_blank() {
					context.declare_type(id, Some(loc.clone()));
				}

				let types_list =
					context.try_create_list_with::<Error<F>, _, _>(types, |ty_expr, context| {
						let Loc(id, loc) = ty_expr.build(local_context, context)?;
						Ok(Caused::new(id.into_term(), Some(loc)))
					})?;

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
	) -> Result<Self::Target, Error<F>> {
		self.into_value().expr.build(local_context, context)
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::InnerTypeExpr<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
	) -> Result<Self::Target, Error<F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::InnerTypeExpr::Id(id) => {
				if let Some(Loc(id, id_loc)) = local_context.next_id.take() {
					return Err(Loc(LocalError::TypeAlias(id, id_loc), loc).into());
				}

				id.build(local_context, context)
			}
			crate::InnerTypeExpr::Reference(r) => r.build(local_context, context),
			crate::InnerTypeExpr::Literal(lit) => {
				local_context.insert_literal_type(context, Loc(lit, loc))
			}
			crate::InnerTypeExpr::PropertyRestriction(_) => {
				todo!()
			}
		}
	}
}

impl<F: Clone + Ord> Declare<F> for Loc<crate::PropertyDefinition<F>, F> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
	) -> Result<(), Error<F>> {
		let Loc(id, _) = self.id.clone().build(local_context, context)?;
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
	) -> Result<Self::Target, Error<F>> {
		let Loc(def, loc) = self;
		let Loc(id, id_loc) = def.id.build(local_context, context)?;

		let doc = def
			.doc
			.map(|doc| doc.build(local_context, context))
			.transpose()?;

		let mut functional = true;
		let mut functional_loc = None;
		let mut required = false;
		let mut required_loc = None;

		let range = def
			.ty
			.map(|Loc(ty, _)| -> Result<_, Error<F>> {
				let scope = local_context.scope.take();
				let range = ty.expr.build(local_context, context)?;
				local_context.scope = scope;

				for Loc(ann, ann_loc) in ty.annotations {
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
		if let Some(Loc(range, range_loc)) = range {
			prop.set_range(range, Some(range_loc))?;
		}
		prop.set_functional(functional, functional_loc)?;
		prop.set_required(required, required_loc)?;

		Ok(Loc(id, id_loc))
	}
}

impl<F: Clone + Ord> Declare<F> for Loc<crate::LayoutDefinition<F>, F> {
	fn declare(
		&self,
		local_context: &mut LocalContext<F>,
		context: &mut Context<F, Descriptions>,
	) -> Result<(), Error<F>> {
		let Loc(id, _) = self.id.clone().build(local_context, context)?;
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
	) -> Result<(), Error<F>> {
		let Loc(def, _) = self;
		let Loc(id, id_loc) = def.id.build(local_context, context)?;

		if let Some(doc) = def.doc {
			let (label, doc) = doc.build(local_context, context)?;
			let node = context.get_mut(id).unwrap();

			if let Some(label) = label {
				node.add_label(label);
			}

			if let Some(doc) = doc {
				node.documentation_mut().add(doc);
			}
		}

		let Loc(ty_id, ty_id_loc) = def.ty_id.build(local_context, context)?;
		context
			.get_mut(id)
			.unwrap()
			.as_layout_mut()
			.unwrap()
			.set_type(ty_id, Some(ty_id_loc))?;

		match def.description {
			Loc(crate::LayoutDescription::Normal(fields), fields_loc) => {
				let fields_list =
					context.try_create_list_with::<Error<F>, _, _>(fields, |field, context| {
						local_context.scope = Some(ty_id);
						let Loc(item, item_loc) = field.build(local_context, context)?;
						local_context.scope = None;
						Ok(Caused::new(item.into_term(), Some(item_loc)))
					})?;

				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.set_fields(fields_list, Some(fields_loc))?;
			}
			Loc(crate::LayoutDescription::Alias(expr), expr_loc) => {
				local_context.next_id = Some(Loc(id, id_loc));
				Loc(expr, expr_loc).build(local_context, context)?;
				local_context.next_id = None;
			}
		}

		Ok(())
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::Alias, F> {
	type Target = Loc<treeldr::vocab::Name, F>;

	fn build(
		self,
		_local_context: &mut LocalContext<F>,
		_context: &mut Context<F, Descriptions>,
	) -> Result<Self::Target, Error<F>> {
		let Loc(name, loc) = self;
		match treeldr::vocab::Name::new(name.as_str()) {
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
	) -> Result<Self::Target, Error<F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::OuterLayoutExpr::Inner(e) => Loc(e, loc).build(local_context, context),
			crate::OuterLayoutExpr::Union(label, options) => {
				let Loc(id, _) =
					local_context.anonymous_id(Some(label), context.vocabulary_mut(), loc.clone());
				if id.is_blank() {
					context.declare_layout(id, Some(loc.clone()));
				}

				let variants = context.try_create_list_with::<Error<F>, _, _>(
					options,
					|layout_expr, context| {
						let loc = layout_expr.location().clone();
						let variant_id = Id::Blank(context.vocabulary_mut().new_blank_label());

						let (layout_expr, variant_name) = if layout_expr.value().expr.is_namable() {
							let name = layout_expr.value().name.clone();
							(layout_expr, name)
						} else {
							let Loc(layout_expr, loc) = layout_expr;
							let (expr, name) = layout_expr.into_parts();
							(
								Loc(crate::NamedInnerLayoutExpr { expr, name: None }, loc),
								name,
							)
						};

						let Loc(layout, layout_loc) = layout_expr.build(local_context, context)?;

						let variant_name = variant_name
							.map(|name| name.build(local_context, context))
							.transpose()?;

						context.declare_layout_variant(variant_id, Some(loc.clone()));

						let variant = context
							.get_mut(variant_id)
							.unwrap()
							.as_layout_variant_mut()
							.unwrap();
						variant.set_layout(layout, Some(layout_loc))?;

						if let Some(Loc(name, name_loc)) = variant_name {
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
			crate::OuterLayoutExpr::Intersection(label, types) => {
				let Loc(id, _) =
					local_context.anonymous_id(Some(label), context.vocabulary_mut(), loc.clone());
				if id.is_blank() {
					context.declare_layout(id, Some(loc.clone()));
				}

				let layouts_list = context.try_create_list_with::<Error<F>, _, _>(
					types,
					|layout_expr, context| {
						let Loc(id, loc) = layout_expr.build(local_context, context)?;
						Ok(Caused::new(id.into_term(), Some(loc)))
					},
				)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_description(LayoutDescription::Intersection(layouts_list), Some(loc.clone()))?;
				if local_context.implicit_definition {
					layout.set_type(id, Some(loc.clone()))?;
				}

				Ok(Loc(id, loc))
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
	) -> Result<Self::Target, Error<F>> {
		let Loc(this, loc) = self;
		let is_namable = this.expr.is_namable();
		let Loc(id, expr_loc) = this.expr.build(local_context, context)?;

		if let Some(name) = this.name {
			let Loc(name, name_loc) = name.build(local_context, context)?;
			if is_namable {
				context
					.get_mut(id)
					.unwrap()
					.as_layout_mut()
					.unwrap()
					.set_name(name, Some(name_loc))?;
			} else {
				return Err(Loc(LocalError::LayoutAlias(id, expr_loc), loc).into());
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
	) -> Result<Self::Target, Error<F>> {
		let Loc(expr, loc) = self;

		match expr {
			crate::InnerLayoutExpr::Id(id) => {
				if let Some(Loc(id, id_loc)) = local_context.next_id.take() {
					return Err(Loc(LocalError::LayoutAlias(id, id_loc), loc).into());
				}

				id.build(local_context, context)
			}
			crate::InnerLayoutExpr::Reference(r) => {
				let Loc(id, _) =
					local_context.anonymous_id(None, context.vocabulary_mut(), loc.clone());
				if id.is_blank() {
					context.declare_layout(id, Some(loc.clone()));
				}

				let Loc(deref_layout, deref_loc) = r.build(local_context, context)?;

				let layout = context.get_mut(id).unwrap().as_layout_mut().unwrap();
				layout.set_deref_to(deref_layout, Some(deref_loc))?;

				Ok(Loc(id, loc))
			}
			crate::InnerLayoutExpr::Literal(lit) => {
				local_context.insert_literal_layout(context, Loc(lit, loc))
			}
			crate::InnerLayoutExpr::FieldRestriction(_) => {
				todo!()
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
	) -> Result<Self::Target, Error<F>> {
		let Loc(def, loc) = self;

		let id = Id::Blank(context.vocabulary_mut().new_blank_label());

		let Loc(prop_id, prop_id_loc) = def.id.build(local_context, context)?;

		let Loc(name, name_loc) = def
			.alias
			.unwrap_or_else(|| match prop_id {
				Id::Iri(id) => Loc(
					crate::Alias(
						id.iri(context.vocabulary())
							.unwrap()
							.path()
							.file_name()
							.expect("invalid property IRI")
							.to_owned(),
					),
					prop_id_loc.clone(),
				),
				_ => panic!("invalid property IRI"),
			})
			.build(local_context, context)?;

		let mut functional = true;
		let mut functional_loc = None;
		let mut required = true;
		let mut required_loc = None;

		let Loc(layout, layout_loc) = match def.layout {
			Some(Loc(layout, _)) => {
				let scope = local_context.scope.take();
				let layout_id = layout.expr.build(local_context, context)?;
				local_context.scope = scope;

				for Loc(ann, ann_loc) in layout.annotations {
					match ann {
						crate::Annotation::Multiple => {
							functional = false;
							functional_loc = Some(ann_loc)
						}
						crate::Annotation::Required => {
							required = true;
							required_loc = Some(ann_loc)
						}
					}
				}

				layout_id
			}
			None => todo!("infer field layout"),
		};

		let doc = def
			.doc
			.map(|doc| doc.build(local_context, context))
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
		field.set_layout(layout, Some(layout_loc))?;
		field.set_functional(functional, functional_loc)?;
		field.set_required(required, required_loc)?;

		Ok(Loc(id, loc))
	}
}

#[derive(PartialEq, Eq)]
pub enum LayoutDescription {
	/// Standard layout description.
	Standard(treeldr_build::layout::Description),

	Intersection(Id),

	/// Pseudo layout definition.
	Restriction,
}

impl From<treeldr_build::layout::Description> for LayoutDescription {
	fn from(d: treeldr_build::layout::Description) -> Self {
		Self::Standard(d)
	}
}

impl<F: Clone + Ord> treeldr_build::layout::PseudoDescription<F> for LayoutDescription {
	type Error = Error<F>;

	fn as_standard(&self) -> Option<&treeldr_build::layout::Description> {
		match self {
			Self::Standard(s) => Some(s),
			_ => None,
		}
	}

	fn sub_layouts<D: treeldr_build::Descriptions<F>>(
		&self,
		context: &Context<F, D>,
		causes: &Causes<F>,
	) -> Result<Vec<SubLayout<F>>, Self::Error> {
		match self {
			Self::Standard(s) => Ok(s.sub_layouts(context, causes)?),
			Self::Intersection(_) => Ok(Vec::new()),
			Self::Restriction => todo!(),
		}
	}

	fn dependencies(
		&self,
		id: Id,
		nodes: &context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<Item<F>>, Error<F>> {
		match self {
			Self::Standard(s) => Ok(s.dependencies(id, nodes, causes)?),
			Self::Intersection(layout_list_id) => {
				let layouts = nodes
					.require_list(*layout_list_id, causes.preferred().cloned())?
					.iter(nodes)
					.map(|item| -> Result<_, Error<F>> {
						let (object, causes) = item?.clone().into_parts();
						let layout_id = match object {
							Object::Literal(lit) => return Err(treeldr_build::Error::new(
								treeldr_build::error::LiteralUnexpected(lit).into(),
								causes.preferred().cloned(),
							).into()),
							Object::Iri(id) => Id::Iri(id),
							Object::Blank(id) => Id::Blank(id),
						};

						Ok(Item::Layout(
							**nodes.require_layout(layout_id, causes.into_preferred())?,
						))
					})
					.try_collect()?;
				Ok(layouts)
			}
			_ => todo!(),
		}
	}

	fn build(
		self,
		id: Id,
		mut name: MaybeSet<Name, F>,
		nodes: &context::AllocatedNodes<F>,
		dependencies: Dependencies<F>,
		causes: &Causes<F>,
	) -> Result<treeldr::layout::Description<F>, Self::Error> {
		match self {
			Self::Standard(s) => Ok(s.build(id, name, nodes, dependencies, causes)?),
			Self::Intersection(layout_list_id) => {
				use treeldr_build::layout::ComputeIntersection;
				let layouts = nodes
					.require_list(layout_list_id, causes.preferred().cloned())?
					.iter(nodes)
					.map(|item| {
						let (object, causes) = item?.clone().into_parts();
						let layout_id = match object {
							Object::Literal(lit) => return Err(Caused::new(
								treeldr_build::error::LiteralUnexpected(lit).into(),
								causes.preferred().cloned(),
							)),
							Object::Iri(id) => Id::Iri(id),
							Object::Blank(id) => Id::Blank(id),
						};

						let layout_ref =
							nodes.require_layout(layout_id, causes.into_preferred())?;
						Ok(dependencies.layouts[layout_ref.index()].as_ref().unwrap())
					});

				let mut desc: Option<treeldr::layout::Description<F>> = None;
				for layout in layouts {
					let layout = layout?;

					desc = Some(match desc {
						Some(desc) => desc.intersected_with(
							id,
							layout.description_with_causes(),
							name.take(),
							dependencies.layouts,
						)?,
						None => layout.description().clone(),
					})
				}

				Ok(desc.unwrap())
			}
			_ => todo!(),
		}
	}
}
