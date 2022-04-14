use iref::{IriBuf, IriRef, IriRefBuf};
use locspan::{Loc, Location};
use rdf_types::{loc::Literal, Quad};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::vocab::*;

pub trait Build<F> {
	type Target;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>>;
}

#[derive(Debug)]
pub enum Error<F> {
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

impl<'c, 't, F: Clone> crate::reporting::Diagnose<F> for Loc<Error<F>, F> {
	fn message(&self) -> String {
		match self.value() {
			Error::InvalidExpandedCompactIri(_) => "invalid expanded compact IRI".to_string(),
			Error::UndefinedPrefix(_) => "undefined prefix".to_string(),
			Error::AlreadyDefinedPrefix(_, _) => "already defined prefix".to_string(),
			Error::NoBaseIri => "no base IRI".to_string(),
			Error::BaseIriMismatch { .. } => "base IRI mismatch".to_string(),
			Error::TypeAlias(_, _) => "type aliases are not supported".to_string(),
			Error::LayoutAlias(_, _) => "layout aliases are not supported".to_string(),
		}
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		let mut labels = vec![self
			.location()
			.clone()
			.into_primary_label()
			.with_message(self.to_string())];

		match self.value() {
			Error::AlreadyDefinedPrefix(_, original_loc) => labels.push(
				original_loc
					.clone()
					.into_secondary_label()
					.with_message("original prefix defined here".to_string()),
			),
			Error::BaseIriMismatch { because, .. } => labels.push(
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

impl<F> fmt::Display for Error<F> {
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

/// Build context.
pub struct Context<'v, F> {
	/// Base IRI of th parsed document.
	base_iri: Option<IriBuf>,

	/// Vocabulary.
	vocabulary: &'v mut Vocabulary,

	/// Bound prefixes.
	prefixes: HashMap<String, Loc<IriBuf, F>>,

	/// Current scope.
	scope: Option<Term>,

	next_id: Option<Loc<Id, F>>,

	/// Associates each literal type/value to a blank node label.
	literal: BTreeMap<Loc<crate::Literal, F>, Loc<Id, F>>,

	/// Associates each union type (location) to a blank node label.
	anonymous_types: BTreeMap<Location<F>, Loc<Id, F>>,
}

impl<'v, F> Context<'v, F> {
	pub fn new(vocabulary: &'v mut Vocabulary, base_iri: Option<IriBuf>) -> Self {
		Self {
			base_iri,
			vocabulary,
			prefixes: HashMap::new(),
			scope: None,
			next_id: None,
			literal: BTreeMap::new(),
			anonymous_types: BTreeMap::new(),
		}
	}

	pub fn vocabulary(&self) -> &Vocabulary {
		self.vocabulary
	}

	pub fn into_vocabulary(self) -> &'v mut Vocabulary {
		self.vocabulary
	}

	pub fn next_id(&mut self, loc: Location<F>) -> Loc<Id, F> {
		self.next_id
			.take()
			.unwrap_or_else(|| Loc(Id::Blank(self.vocabulary.new_blank_label()), loc))
	}

	/// Inserts a new literal type & layout.
	pub fn insert_literal(
		&mut self,
		quads: &mut Vec<LocQuad<F>>,
		lit: Loc<crate::Literal, F>,
	) -> Loc<Id, F>
	where
		F: Clone + Ord,
	{
		use std::collections::btree_map::Entry;
		match self.literal.entry(lit) {
			Entry::Occupied(entry) => {
				self.next_id.take();
				entry.get().clone()
			}
			Entry::Vacant(entry) => {
				let loc = entry.key().location();
				let id = self.next_id.take().unwrap_or_else(|| {
					Loc(Id::Blank(self.vocabulary.new_blank_label()), loc.clone())
				});

				if id.is_blank() {
					// Define the type.
					quads.push(Loc(
						Quad(
							id.clone(),
							Loc(Term::Rdf(Rdf::Type), loc.clone()),
							Loc(Object::Iri(Term::Rdfs(Rdfs::Class)), loc.clone()),
							None,
						),
						loc.clone(),
					));

					// Define the associated layout.
					quads.push(Loc(
						Quad(
							id.clone(),
							Loc(Term::Rdf(Rdf::Type), loc.clone()),
							Loc(Object::Iri(Term::TreeLdr(TreeLdr::Layout)), loc.clone()),
							None,
						),
						loc.clone(),
					));

					quads.push(Loc(
						Quad(
							id.clone(),
							Loc(Term::TreeLdr(TreeLdr::LayoutFor), loc.clone()),
							Loc(id.value().into_term(), loc.clone()),
							None,
						),
						loc.clone(),
					));
				}

				match entry.key().value() {
					crate::Literal::String(s) => {
						quads.push(Loc(
							Quad(
								id.clone(),
								Loc(Term::TreeLdr(TreeLdr::Singleton), loc.clone()),
								Loc(
									Object::Literal(Literal::String(Loc(
										s.clone().into(),
										loc.clone(),
									))),
									loc.clone(),
								),
								None,
							),
							loc.clone(),
						));
					}
					crate::Literal::RegExp(e) => {
						quads.push(Loc(
							Quad(
								id.clone(),
								Loc(Term::TreeLdr(TreeLdr::Matches), loc.clone()),
								Loc(
									Object::Literal(Literal::String(Loc(
										e.clone().into(),
										loc.clone(),
									))),
									loc.clone(),
								),
								None,
							),
							loc.clone(),
						));
					}
				}

				entry.insert(id.clone());
				id
			}
		}
	}

	fn generate_union_type(
		&mut self,
		id: Loc<Id, F>,
		quads: &mut Vec<LocQuad<F>>,
		Loc(options, loc): Loc<Vec<Loc<crate::NamedInnerTypeExpr<F>, F>>, F>,
	) -> Result<(), Loc<Error<F>, F>>
	where
		F: Clone + Ord,
	{
		let options_list = options.into_iter().try_into_rdf_list(
			self,
			quads,
			loc.clone(),
			|ty_expr, ctx, quads| ty_expr.build(ctx, quads),
		)?;

		if id.is_blank() {
			quads.push(Loc(
				Quad(
					id.clone(),
					Loc(Term::Rdf(Rdf::Type), loc.clone()),
					Loc(Object::Iri(Term::Rdfs(Rdfs::Class)), loc.clone()),
					None,
				),
				loc.clone(),
			));
		}

		quads.push(Loc(
			Quad(
				id,
				Loc(Term::Owl(Owl::UnionOf), options_list.location().clone()),
				options_list,
				None,
			),
			loc,
		));

		Ok(())
	}

	fn generate_union_layout(
		&mut self,
		id: Loc<Id, F>,
		quads: &mut Vec<LocQuad<F>>,
		Loc(options, loc): Loc<Vec<Loc<crate::NamedInnerLayoutExpr<F>, F>>, F>,
	) -> Result<(), Loc<Error<F>, F>>
	where
		F: Clone + Ord,
	{
		let variants_list = options.into_iter().try_into_rdf_list(
			self,
			quads,
			loc.clone(),
			|ty_expr, ctx, quads| {
				let loc = ty_expr.location().clone();
				let variant_label = ctx.vocabulary.new_blank_label();
				let variant_name = ty_expr.name.clone();
				let ty = ty_expr.build(ctx, quads)?;

				quads.push(Loc(
					Quad(
						Loc(Id::Blank(variant_label), loc.clone()),
						Loc(Term::Rdf(Rdf::Type), loc.clone()),
						Loc(Object::Iri(Term::TreeLdr(TreeLdr::Variant)), loc.clone()),
						None,
					),
					loc.clone(),
				));
				quads.push(Loc(
					Quad(
						Loc(Id::Blank(variant_label), loc.clone()),
						Loc(Term::TreeLdr(TreeLdr::Format), loc.clone()),
						ty,
						None,
					),
					loc.clone(),
				));

				if let Some(Loc(name, name_loc)) = variant_name {
					quads.push(Loc(
						Quad(
							Loc(Id::Blank(variant_label), loc.clone()),
							Loc(Term::TreeLdr(TreeLdr::Name), loc.clone()),
							Loc(
								Object::Literal(Literal::String(Loc(
									name.into_string().into(),
									name_loc.clone(),
								))),
								name_loc,
							),
							None,
						),
						loc.clone(),
					));
				}

				Ok(Loc(Object::Blank(variant_label), loc))
			},
		)?;

		if id.is_blank() {
			quads.push(Loc(
				Quad(
					id.clone(),
					Loc(Term::Rdf(Rdf::Type), loc.clone()),
					Loc(Object::Iri(Term::TreeLdr(TreeLdr::Layout)), loc.clone()),
					None,
				),
				loc.clone(),
			));
			quads.push(Loc(
				Quad(
					id.clone(),
					Loc(Term::TreeLdr(TreeLdr::LayoutFor), loc.clone()),
					Loc(id.into_term(), loc.clone()),
					None,
				),
				loc.clone(),
			));
		}

		quads.push(Loc(
			Quad(
				id,
				Loc(Term::TreeLdr(TreeLdr::Enumeration), loc.clone()),
				variants_list,
				None,
			),
			loc,
		));

		Ok(())
	}

	fn generate_intersection_type(
		&mut self,
		id: Loc<Id, F>,
		quads: &mut Vec<LocQuad<F>>,
		Loc(types, loc): Loc<Vec<Loc<crate::NamedInnerTypeExpr<F>, F>>, F>,
	) -> Result<(), Loc<Error<F>, F>>
	where
		F: Clone + Ord,
	{
		let types_list = types.into_iter().try_into_rdf_list(
			self,
			quads,
			loc.clone(),
			|ty_expr, ctx, quads| ty_expr.build(ctx, quads),
		)?;

		if id.is_blank() {
			quads.push(Loc(
				Quad(
					id.clone(),
					Loc(Term::Rdf(Rdf::Type), loc.clone()),
					Loc(Object::Iri(Term::Rdfs(Rdfs::Class)), loc.clone()),
					None,
				),
				loc.clone(),
			));
		}

		quads.push(Loc(
			Quad(
				id,
				Loc(
					Term::Owl(Owl::IntersectionOf),
					types_list.location().clone(),
				),
				types_list,
				None,
			),
			loc,
		));

		Ok(())
	}

	fn generate_intersection_layout(
		&mut self,
		id: Loc<Id, F>,
		quads: &mut Vec<LocQuad<F>>,
		Loc(layouts, loc): Loc<Vec<Loc<crate::NamedInnerLayoutExpr<F>, F>>, F>,
	) -> Result<(), Loc<Error<F>, F>>
	where
		F: Clone + Ord,
	{
		let layouts_list = layouts.into_iter().try_into_rdf_list(
			self,
			quads,
			loc.clone(),
			|layout_expr, ctx, quads| layout_expr.build(ctx, quads),
		)?;

		if id.is_blank() {
			quads.push(Loc(
				Quad(
					id.clone(),
					Loc(Term::Rdf(Rdf::Type), loc.clone()),
					Loc(Object::Iri(Term::TreeLdr(TreeLdr::Layout)), loc.clone()),
					None,
				),
				loc.clone(),
			));
		}

		quads.push(Loc(
			Quad(
				id,
				Loc(
					Term::TreeLdr(TreeLdr::Intersection),
					layouts_list.location().clone(),
				),
				layouts_list,
				None,
			),
			loc,
		));

		Ok(())
	}

	pub fn insert_anonymous_type(&mut self, loc: Location<F>) -> Loc<Id, F>
	where
		F: Clone + Ord,
	{
		self.next_id.take().unwrap_or_else(|| {
			self.anonymous_types
				.entry(loc.clone())
				.or_insert_with(|| Loc(Id::Blank(self.vocabulary.new_blank_label()), loc))
				.clone()
		})
	}
}

impl<'v, F: Clone> Context<'v, F> {
	pub fn base_iri(&self, loc: Location<F>) -> Result<IriBuf, Loc<Error<F>, F>> {
		match &self.scope {
			Some(scope) => {
				let mut iri = scope.iri(self.vocabulary).unwrap().to_owned();
				iri.path_mut().open();
				Ok(iri)
			}
			None => self.base_iri.clone().ok_or(Loc(Error::NoBaseIri, loc)),
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
	) -> Result<(), Loc<Error<F>, F>> {
		use std::collections::hash_map::Entry;
		match self.prefixes.entry(prefix) {
			Entry::Occupied(entry) => Err(Loc(
				Error::AlreadyDefinedPrefix(entry.key().to_owned(), entry.get().location().clone()),
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
	) -> Result<IriBuf, Loc<Error<F>, F>> {
		match self.prefixes.get(prefix) {
			Some(iri) => match IriBuf::try_from(iri.as_str().to_string() + iri_ref.as_str()) {
				Ok(iri) => Ok(iri),
				Err((_, string)) => Err(Loc(Error::InvalidExpandedCompactIri(string), loc.clone())),
			},
			None => Err(Loc(Error::UndefinedPrefix(prefix.to_owned()), loc.clone())),
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::Document<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let Loc(doc, _) = self;

		let mut declared_base_iri = None;
		for Loc(base_iri, loc) in doc.bases {
			match declared_base_iri.take() {
				Some(Loc(declared_base_iri, d_loc)) => {
					if declared_base_iri != base_iri {
						return Err(Loc(
							Error::BaseIriMismatch {
								expected: Box::new(declared_base_iri),
								found: Box::new(base_iri),
								because: d_loc,
							},
							loc,
						));
					}
				}
				None => {
					ctx.set_base_iri(base_iri.clone());
					declared_base_iri = Some(Loc(base_iri, loc));
				}
			}
		}

		for import in doc.uses {
			import.build(ctx, quads)?
		}

		for ty in doc.types {
			ty.build(ctx, quads)?
		}

		for layout in doc.layouts {
			layout.build(ctx, quads)?
		}

		Ok(())
	}
}

impl<F: Clone> Build<F> for Loc<crate::Id, F> {
	type Target = Loc<Term, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		_quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(id, loc) = self;
		let iri = match id {
			crate::Id::Name(name) => {
				let mut iri_ref = IriRefBuf::from_string(name).unwrap();
				iri_ref.resolve(ctx.base_iri(loc.clone())?.as_iri());
				iri_ref.try_into().unwrap()
			}
			crate::Id::IriRef(iri_ref) => iri_ref.resolved(ctx.base_iri(loc.clone())?.as_iri()),
			crate::Id::Compact(prefix, iri_ref) => {
				ctx.expand_compact_iri(&prefix, iri_ref.as_iri_ref(), &loc)?
			}
		};

		Ok(Loc(Term::from_iri(iri, ctx.vocabulary), loc))
	}
}

impl<F: Clone> Build<F> for Loc<crate::Use<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		_quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let Loc(import, loc) = self;
		ctx.declare_prefix(
			import.prefix.into_value().into_string(),
			import.iri.into_value(),
			loc,
		)
	}
}

fn build_doc<F: Clone>(
	Loc(doc, loc): Loc<crate::Documentation<F>, F>,
	subject: Loc<Id, F>,
	quads: &mut Vec<LocQuad<F>>,
) {
	let mut label = String::new();
	let mut label_loc = loc.clone();

	let mut description = String::new();
	let mut description_loc = loc.clone();

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

	if !label.is_empty() {
		quads.push(Loc(
			Quad(
				subject.clone(),
				Loc(Term::Rdfs(Rdfs::Label), loc.clone()),
				Loc(
					Object::Literal(Literal::String(Loc(label.into(), label_loc.clone()))),
					label_loc,
				),
				None,
			),
			loc.clone(),
		))
	}

	if !description.is_empty() {
		quads.push(Loc(
			Quad(
				subject,
				Loc(Term::Rdfs(Rdfs::Comment), loc.clone()),
				Loc(
					Object::Literal(Literal::String(Loc(
						description.into(),
						description_loc.clone(),
					))),
					description_loc,
				),
				None,
			),
			loc,
		))
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::TypeDefinition<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let implicit_layout = Loc(self.implicit_layout_definition(), self.location().clone());
		let Loc(def, _) = self;
		let Loc(id, id_loc) = def.id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Term::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Term::Rdfs(Rdfs::Class)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		match def.description {
			Loc(crate::TypeDescription::Normal(properties), _) => {
				for property in properties {
					ctx.scope = Some(id);
					let Loc(prop, prop_loc) = property.build(ctx, quads)?;
					ctx.scope = None;

					quads.push(Loc(
						Quad(
							Loc(Id::Iri(prop), prop_loc.clone()),
							Loc(Term::Rdfs(Rdfs::Domain), prop_loc.clone()),
							Loc(Object::Iri(id), id_loc.clone()),
							None,
						),
						prop_loc,
					));
				}
			}
			Loc(crate::TypeDescription::Alias(expr), expr_loc) => {
				ctx.next_id = Some(Loc(Id::Iri(id), id_loc.clone()));
				Loc(expr, expr_loc).build(ctx, quads)?;
				ctx.next_id = None
			}
		}

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Id::Iri(id), id_loc), quads)
		}

		implicit_layout.build(ctx, quads)
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::PropertyDefinition<F>, F> {
	type Target = Loc<Term, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(def, _) = self;
		let Loc(id, id_loc) = def.id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Term::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Term::Rdf(Rdf::Property)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Id::Iri(id), id_loc.clone()), quads)
		}

		if let Some(Loc(ty, _)) = def.ty {
			let scope = ctx.scope.take();
			let object = ty.expr.build(ctx, quads)?;
			let object_loc = object.location().clone();
			ctx.scope = scope;

			quads.push(Loc(
				Quad(
					Loc(Id::Iri(id), id_loc.clone()),
					Loc(Term::Rdfs(Rdfs::Range), object_loc.clone()),
					object,
					None,
				),
				object_loc,
			));

			for Loc(ann, ann_loc) in ty.annotations {
				match ann {
					crate::Annotation::Multiple => quads.push(Loc(
						Quad(
							Loc(Id::Iri(id), id_loc.clone()),
							Loc(Term::Schema(Schema::MultipleValues), ann_loc.clone()),
							Loc(Object::Iri(Term::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
					crate::Annotation::Required => quads.push(Loc(
						Quad(
							Loc(Id::Iri(id), id_loc.clone()),
							Loc(Term::Schema(Schema::ValueRequired), ann_loc.clone()),
							Loc(Object::Iri(Term::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
				}
			}
		}

		Ok(Loc(id, id_loc))
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::OuterTypeExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::OuterTypeExpr::Inner(e) => Loc(e, loc).build(ctx, quads),
			crate::OuterTypeExpr::Union(options) => {
				let id = ctx.insert_anonymous_type(loc.clone());
				ctx.generate_union_type(id.clone(), quads, Loc(options, loc.clone()))?;
				Ok(Loc(id.into_value().into_term(), loc))
			}
			crate::OuterTypeExpr::Intersection(types) => {
				let id = ctx.insert_anonymous_type(loc.clone());
				ctx.generate_intersection_type(id.clone(), quads, Loc(types, loc.clone()))?;
				Ok(Loc(id.into_value().into_term(), loc))
			}
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::NamedInnerTypeExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		self.into_value().expr.build(ctx, quads)
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::InnerTypeExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::InnerTypeExpr::Id(id) => {
				if let Some(Loc(id, id_loc)) = ctx.next_id.take() {
					return Err(Loc(Error::TypeAlias(id, id_loc), loc));
				}

				let Loc(id, _) = id.build(ctx, quads)?;
				Ok(Loc(Object::Iri(id), loc))
			}
			crate::InnerTypeExpr::Reference(r) => r.build(ctx, quads),
			crate::InnerTypeExpr::Literal(lit) => {
				let id = ctx.insert_literal(quads, Loc(lit, loc.clone()));
				Ok(Loc(id.into_value().into_term(), loc))
			}
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::LayoutDefinition<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let Loc(def, _) = self;
		let Loc(id, id_loc) = def.id.build(ctx, quads)?;
		let Loc(ty_id, ty_id_loc) = def.ty_id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Term::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Term::TreeLdr(TreeLdr::Layout)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		match def.description {
			Loc(crate::LayoutDescription::Normal(fields), fields_loc) => {
				let fields_list = fields.into_iter().try_into_rdf_list(
					ctx,
					quads,
					fields_loc,
					|field, ctx, quads| {
						ctx.scope = Some(ty_id);
						let item = field.build(ctx, quads)?;
						ctx.scope = None;
						Ok(item)
					},
				)?;

				quads.push(Loc(
					Quad(
						Loc(Id::Iri(id), id_loc.clone()),
						Loc(Term::TreeLdr(TreeLdr::Fields), id_loc.clone()),
						fields_list,
						None,
					),
					id_loc.clone(),
				));
			}
			Loc(crate::LayoutDescription::Alias(expr), expr_loc) => {
				ctx.next_id = Some(Loc(Id::Iri(id), id_loc.clone()));
				Loc(expr, expr_loc).build(ctx, quads)?;
				ctx.next_id = None;
			}
		}

		if let Some(iri) = id.iri(ctx.vocabulary()) {
			if let Some(name) = iri.path().file_name() {
				if let Ok(name) = Name::new(name) {
					quads.push(Loc(
						Quad(
							Loc(Id::Iri(id), id_loc.clone()),
							Loc(Term::TreeLdr(TreeLdr::Name), id_loc.clone()),
							Loc(
								Object::Literal(Literal::String(Loc(
									name.to_string().into(),
									id_loc.clone(),
								))),
								id_loc.clone(),
							),
							None,
						),
						id_loc.clone(),
					));
				}
			}
		}

		let for_loc = id_loc.clone().with(ty_id_loc.span());
		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Term::TreeLdr(TreeLdr::LayoutFor), id_loc.clone()),
				Loc(Object::Iri(ty_id), ty_id_loc),
				None,
			),
			for_loc,
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Id::Iri(id), id_loc), quads)
		}

		Ok(())
	}
}

pub trait TryIntoRdfList<F, T> {
	fn try_into_rdf_list<E, K>(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
		loc: Location<F>,
		f: K,
	) -> Result<Loc<Object<F>, F>, E>
	where
		K: FnMut(T, &mut Context<F>, &mut Vec<LocQuad<F>>) -> Result<Loc<Object<F>, F>, E>;
}

impl<F: Clone, I: DoubleEndedIterator> TryIntoRdfList<F, I::Item> for I {
	fn try_into_rdf_list<E, K>(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
		loc: Location<F>,
		mut f: K,
	) -> Result<Loc<Object<F>, F>, E>
	where
		K: FnMut(I::Item, &mut Context<F>, &mut Vec<LocQuad<F>>) -> Result<Loc<Object<F>, F>, E>,
	{
		let mut head = Loc(Object::Iri(Term::Rdf(Rdf::Nil)), loc);
		for item in self.rev() {
			let item = f(item, ctx, quads)?;
			let item_label = ctx.vocabulary.new_blank_label();
			let item_loc = item.location().clone();
			let list_loc = head.location().clone().with(item_loc.span());

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), list_loc.clone()),
					Loc(Term::Rdf(Rdf::Type), list_loc.clone()),
					Loc(Object::Iri(Term::Rdf(Rdf::List)), list_loc.clone()),
					None,
				),
				item_loc.clone(),
			));

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), item_loc.clone()),
					Loc(Term::Rdf(Rdf::First), item_loc.clone()),
					item,
					None,
				),
				item_loc.clone(),
			));

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), head.location().clone()),
					Loc(Term::Rdf(Rdf::Rest), head.location().clone()),
					head,
					None,
				),
				item_loc.clone(),
			));

			head = Loc(Object::Blank(item_label), list_loc);
		}

		Ok(head)
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::FieldDefinition<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(def, loc) = self;

		let label = ctx.vocabulary.new_blank_label();
		let Loc(prop_id, prop_id_loc) = def.id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Id::Blank(label), loc.clone()),
				Loc(Term::Rdf(Rdf::Type), loc.clone()),
				Loc(Object::Iri(Term::TreeLdr(TreeLdr::Field)), loc.clone()),
				None,
			),
			loc.clone(),
		));

		quads.push(Loc(
			Quad(
				Loc(Id::Blank(label), prop_id_loc.clone()),
				Loc(Term::TreeLdr(TreeLdr::FieldFor), prop_id_loc.clone()),
				Loc(Object::Iri(prop_id), prop_id_loc.clone()),
				None,
			),
			prop_id_loc.clone(),
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Id::Blank(label), prop_id_loc.clone()), quads)
		}

		let Loc(name, name_loc) = match def.alias {
			Some(Loc(alias, alias_loc)) => Loc(alias.into_string(), alias_loc),
			None => Loc(
				prop_id
					.iri(ctx.vocabulary)
					.unwrap()
					.path()
					.file_name()
					.expect("invalid property IRI")
					.to_owned(),
				prop_id_loc.clone(),
			),
		};

		quads.push(Loc(
			Quad(
				Loc(Id::Blank(label), prop_id_loc.clone()),
				Loc(Term::TreeLdr(TreeLdr::Name), prop_id_loc.clone()),
				Loc(
					Object::Literal(Literal::String(Loc(name.into(), name_loc.clone()))),
					name_loc.clone(),
				),
				None,
			),
			name_loc,
		));

		if let Some(Loc(layout, _)) = def.layout {
			let scope = ctx.scope.take();
			let object = layout.expr.build(ctx, quads)?;
			let object_loc = object.location().clone();
			ctx.scope = scope;
			quads.push(Loc(
				Quad(
					Loc(Id::Blank(label), prop_id_loc.clone()),
					Loc(Term::TreeLdr(TreeLdr::Format), object_loc.clone()),
					object,
					None,
				),
				object_loc,
			));

			for Loc(ann, ann_loc) in layout.annotations {
				match ann {
					crate::Annotation::Multiple => quads.push(Loc(
						Quad(
							Loc(Id::Blank(label), prop_id_loc.clone()),
							Loc(Term::Schema(Schema::MultipleValues), ann_loc.clone()),
							Loc(Object::Iri(Term::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
					crate::Annotation::Required => quads.push(Loc(
						Quad(
							Loc(Id::Blank(label), prop_id_loc.clone()),
							Loc(Term::Schema(Schema::ValueRequired), ann_loc.clone()),
							Loc(Object::Iri(Term::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
				}
			}
		}

		Ok(Loc(Object::Blank(label), loc))
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::OuterLayoutExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::OuterLayoutExpr::Inner(e) => Loc(e, loc).build(ctx, quads),
			crate::OuterLayoutExpr::Union(options) => {
				let id = ctx.insert_anonymous_type(loc.clone());
				ctx.generate_union_layout(id.clone(), quads, Loc(options, loc.clone()))?;
				Ok(Loc(id.into_value().into_term(), loc))
			}
			crate::OuterLayoutExpr::Intersection(types) => {
				let id = ctx.insert_anonymous_type(loc.clone());
				ctx.generate_intersection_layout(id.clone(), quads, Loc(types, loc.clone()))?;
				Ok(Loc(id.into_value().into_term(), loc))
			}
		}
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::NamedInnerLayoutExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(this, loc) = self;
		let is_namable = this.expr.is_namable();
		let Loc(object, object_loc) = this.expr.build(ctx, quads)?;

		if let Some(Loc(name, name_loc)) = this.name {
			if is_namable {
				let id = match object {
					rdf_types::Object::Iri(id) => Id::Iri(id),
					rdf_types::Object::Blank(id) => Id::Blank(id),
					_ => unreachable!(),
				};

				quads.push(Loc(
					Quad(
						Loc(id, object_loc),
						Loc(Term::TreeLdr(TreeLdr::Name), loc.clone()),
						Loc(
							Object::Literal(Literal::String(Loc(
								name.into_string().into(),
								name_loc.clone(),
							))),
							name_loc,
						),
						None,
					),
					loc.clone(),
				));
			}
		}

		Ok(Loc(object, loc))
	}
}

impl<F: Clone + Ord> Build<F> for Loc<crate::InnerLayoutExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::InnerLayoutExpr::Id(id) => {
				if let Some(Loc(id, id_loc)) = ctx.next_id.take() {
					return Err(Loc(Error::LayoutAlias(id, id_loc), loc));
				}

				let Loc(id, _) = id.build(ctx, quads)?;
				Ok(Loc(Object::Iri(id), loc))
			}
			crate::InnerLayoutExpr::Reference(r) => {
				let id = ctx.next_id(loc.clone());
				let deref_layout = r.build(ctx, quads)?;

				quads.push(Loc(
					Quad(
						id.clone(),
						Loc(Term::Rdf(Rdf::Type), loc.clone()),
						Loc(Object::Iri(Term::TreeLdr(TreeLdr::Layout)), loc.clone()),
						None,
					),
					loc.clone(),
				));

				quads.push(Loc(
					Quad(
						id.clone(),
						Loc(Term::TreeLdr(TreeLdr::DerefTo), loc.clone()),
						deref_layout,
						None,
					),
					loc.clone(),
				));

				Ok(Loc(id.into_value().into_term(), loc))
			}
			crate::InnerLayoutExpr::Literal(lit) => {
				let id = ctx.insert_literal(quads, Loc(lit, loc.clone()));
				Ok(Loc(id.into_value().into_term(), loc))
			}
		}
	}
}
