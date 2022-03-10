use iref::{Iri, IriBuf, IriRef, IriRefBuf};
use iref_enum::IriEnum;
use locspan::{Loc, Location};
use rdf_types::{loc::Literal, Quad};
use std::{collections::HashMap, fmt};

mod display;

pub use display::{Display, Displayed};

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("tldr" = "https://treeldr.org/")]
pub enum TreeLdr {
	#[iri("tldr:Layout")]
	Layout,

	#[iri("tldr:layoutFor")]
	LayoutFor,

	#[iri("tldr:fields")]
	Fields,

	#[iri("tldr:Field")]
	Field,

	#[iri("tldr:fieldFor")]
	FieldFor,

	#[iri("tldr:Reference")]
	Reference,

	#[iri("tldr:derefTo")]
	DerefTo,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("schema" = "https://schema.org/")]
pub enum Schema {
	#[iri("schema:True")]
	True,

	#[iri("schema:False")]
	False,

	#[iri("schema:multipleValues")]
	MultipleValues,

	#[iri("schema:valueRequired")]
	ValueRequired,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
pub enum Rdfs {
	#[iri("rdfs:Class")]
	Class,

	#[iri("rdfs:label")]
	Label,

	#[iri("rdfs:comment")]
	Comment,

	#[iri("rdfs:domain")]
	Domain,

	#[iri("rdfs:range")]
	Range,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("rdf" = "http://www.w3.org/1999/02/22-rdf-syntax-ns#")]
pub enum Rdf {
	#[iri("rdf:Property")]
	Property,

	#[iri("rdf:List")]
	List,

	#[iri("rdf:type")]
	Type,

	#[iri("rdf:nil")]
	Nil,

	#[iri("rdf:first")]
	First,

	#[iri("rdf:rest")]
	Rest,
}

/// Name index.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Name(usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Id {
	Rdf(Rdf),
	Rdfs(Rdfs),
	Schema(Schema),
	TreeLdr(TreeLdr),
	Name(Name),
}

impl Id {
	pub fn from_iri(iri: IriBuf, ns: &mut Namespace) -> Id {
		match Rdf::try_from(iri.as_iri()) {
			Ok(id) => Id::Rdf(id),
			Err(_) => match Rdfs::try_from(iri.as_iri()) {
				Ok(id) => Id::Rdfs(id),
				Err(_) => match Schema::try_from(iri.as_iri()) {
					Ok(id) => Id::Schema(id),
					Err(_) => match TreeLdr::try_from(iri.as_iri()) {
						Ok(id) => Id::TreeLdr(id),
						Err(_) => Id::Name(ns.insert(iri)),
					},
				},
			},
		}
	}

	pub fn iri<'n>(&self, ns: &'n Namespace) -> Option<Iri<'n>> {
		match self {
			Self::Rdf(id) => Some(id.into()),
			Self::Rdfs(id) => Some(id.into()),
			Self::Schema(id) => Some(id.into()),
			Self::TreeLdr(id) => Some(id.into()),
			Self::Name(name) => ns.get(*name),
		}
	}
}

impl rdf_types::AsTerm for Id {
	type Iri = Self;
	type BlankId = BlankLabel;
	type Literal = rdf_types::Literal;

	fn as_term(&self) -> rdf_types::Term<&Self::Iri, &Self::BlankId, &Self::Literal> {
		rdf_types::Term::Iri(self)
	}
}

impl rdf_types::IntoTerm for Id {
	type Iri = Self;
	type BlankId = BlankLabel;
	type Literal = rdf_types::Literal;

	fn into_term(self) -> rdf_types::Term<Self::Iri, Self::BlankId, Self::Literal> {
		rdf_types::Term::Iri(self)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlankLabel(u32);

impl BlankLabel {
	pub fn new(index: u32) -> Self {
		Self(index)
	}

	pub fn index(&self) -> u32 {
		self.0
	}
}

impl fmt::Display for BlankLabel {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "_:{}", self.index())
	}
}

pub type Subject = rdf_types::Subject<Id, BlankLabel>;

pub type GraphLabel = rdf_types::GraphLabel<Id, BlankLabel>;

pub type Object<F> = rdf_types::Object<Id, BlankLabel, Literal<F>>;

pub type LocQuad<F> = rdf_types::loc::LocQuad<Subject, Id, Object<F>, GraphLabel, F>;

pub type StrippedObject = rdf_types::Object<Id, BlankLabel, rdf_types::Literal>;

pub type StrippedQuad = rdf_types::Quad<Subject, Id, StrippedObject, GraphLabel>;

pub fn strip_quad<F>(Loc(rdf_types::Quad(s, p, o, g), _): LocQuad<F>) -> StrippedQuad {
	use locspan::Strip;
	rdf_types::Quad(
		s.into_value(),
		p.into_value(),
		o.strip(),
		g.map(|g| g.into_value()),
	)
}

pub fn subject_from_rdf(
	subject: rdf_types::Subject,
	ns: &mut Namespace,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> Subject {
	match subject {
		rdf_types::Subject::Iri(iri) => Subject::Iri(Id::from_iri(iri, ns)),
		rdf_types::Subject::Blank(label) => Subject::Blank(blank_label(label)),
	}
}

pub fn object_from_rdf<F>(
	object: rdf_types::loc::Object<F>,
	ns: &mut Namespace,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> Object<F> {
	match object {
		rdf_types::Object::Iri(iri) => Object::Iri(Id::from_iri(iri, ns)),
		rdf_types::Object::Blank(label) => Object::Blank(blank_label(label)),
		rdf_types::Object::Literal(lit) => Object::Literal(lit),
	}
}

pub fn stripped_object_from_rdf(
	object: rdf_types::Object,
	ns: &mut Namespace,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> StrippedObject {
	match object {
		rdf_types::Object::Iri(iri) => StrippedObject::Iri(Id::from_iri(iri, ns)),
		rdf_types::Object::Blank(label) => StrippedObject::Blank(blank_label(label)),
		rdf_types::Object::Literal(lit) => StrippedObject::Literal(lit),
	}
}

pub fn graph_label_from_rdf(
	graph_label: rdf_types::GraphLabel,
	ns: &mut Namespace,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> Subject {
	match graph_label {
		rdf_types::GraphLabel::Iri(iri) => GraphLabel::Iri(Id::from_iri(iri, ns)),
		rdf_types::GraphLabel::Blank(label) => GraphLabel::Blank(blank_label(label)),
	}
}

pub fn loc_quad_from_rdf<F>(
	Loc(rdf_types::Quad(s, p, o, g), loc): rdf_types::loc::LocRdfQuad<F>,
	ns: &mut Namespace,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> LocQuad<F> {
	Loc(
		rdf_types::Quad(
			s.map(|s| subject_from_rdf(s, ns, &mut blank_label)),
			p.map(|p| Id::from_iri(p, ns)),
			o.map(|o| object_from_rdf(o, ns, &mut blank_label)),
			g.map(|g| g.map(|g| graph_label_from_rdf(g, ns, blank_label))),
		),
		loc,
	)
}

pub fn stripped_loc_quad_from_rdf<F>(
	Loc(rdf_types::Quad(s, p, o, g), _): rdf_types::loc::LocRdfQuad<F>,
	ns: &mut Namespace,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> StrippedQuad {
	use locspan::Strip;
	rdf_types::Quad(
		subject_from_rdf(s.into_value(), ns, &mut blank_label),
		Id::from_iri(p.into_value(), ns),
		stripped_object_from_rdf(o.strip(), ns, &mut blank_label),
		g.map(|g| graph_label_from_rdf(g.into_value(), ns, blank_label)),
	)
}

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
	Undefined(crate::Id),
	InvalidExpandedCompactIri(String),
	UndefinedPrefix(String),
	AlreadyDefinedPrefix(String, Location<F>),
}

#[derive(Default)]
pub struct Namespace {
	map: Vec<IriBuf>,
	reverse: HashMap<IriBuf, Name>,
	blank_label_count: u32,
}

impl Namespace {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, name: Name) -> Option<Iri> {
		self.map.get(name.0).map(|iri| iri.as_iri())
	}

	pub fn new_blank_label(&mut self) -> BlankLabel {
		let label = BlankLabel(self.blank_label_count);
		self.blank_label_count += 1;
		label
	}

	pub fn insert(&mut self, iri: IriBuf) -> Name {
		use std::collections::hash_map::Entry;
		match self.reverse.entry(iri) {
			Entry::Occupied(entry) => *entry.get(),
			Entry::Vacant(entry) => {
				let name = Name(self.map.len());
				self.map.push(entry.key().clone());
				entry.insert(name);
				name
			}
		}
	}
}

pub struct Context<F> {
	base_iri: IriBuf,
	namespace: Namespace,
	prefixes: HashMap<String, Loc<IriBuf, F>>,
	scope: Option<Id>,
}

impl<F: Clone> Context<F> {
	pub fn new(base_iri: IriBuf) -> Self {
		Self {
			base_iri,
			namespace: Namespace::new(),
			prefixes: HashMap::new(),
			scope: None,
		}
	}

	pub fn namespace(&self) -> &Namespace {
		&self.namespace
	}

	pub fn into_namespace(self) -> Namespace {
		self.namespace
	}

	pub fn base_iri(&self) -> IriBuf {
		match &self.scope {
			Some(scope) => {
				let mut iri = scope.iri(&self.namespace).unwrap().to_owned();
				iri.path_mut().open();
				iri
			}
			None => self.base_iri.clone(),
		}
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

impl<F: Clone> Build<F> for Loc<crate::Document<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let Loc(doc, _) = self;

		for import in doc.imports {
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
	type Target = Loc<Id, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		_quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(id, loc) = self;
		let iri = match id {
			crate::Id::Name(name) => {
				let mut iri_ref = IriRefBuf::from_string(name).unwrap();
				iri_ref.resolve(ctx.base_iri().as_iri());
				iri_ref.try_into().unwrap()
			}
			crate::Id::IriRef(iri_ref) => iri_ref.resolved(ctx.base_iri().as_iri()),
			crate::Id::Compact(prefix, iri_ref) => {
				ctx.expand_compact_iri(&prefix, iri_ref.as_iri_ref(), &loc)?
			}
		};

		Ok(Loc(Id::from_iri(iri, &mut ctx.namespace), loc))
	}
}

impl<F: Clone> Build<F> for Loc<crate::Import<F>, F> {
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
	subject: Loc<Subject, F>,
	quads: &mut Vec<LocQuad<F>>,
) {
	let mut short = String::new();
	let mut short_loc = loc.clone();

	let mut long = String::new();
	let mut long_loc = loc.clone();

	let mut separated = false;

	for Loc(line, line_loc) in doc.items {
		let line = line.trim();

		if separated {
			if long.is_empty() {
				long_loc = line_loc;
			} else {
				long_loc.span_mut().append(line_loc.span());
			}

			long.push_str(line);
		} else if line.is_empty() {
			separated = true
		} else {
			if short.is_empty() {
				short_loc = line_loc;
			} else {
				short_loc.span_mut().append(line_loc.span());
			}

			short.push_str(line);
		}
	}

	if !long.is_empty() {
		quads.push(Loc(
			Quad(
				subject.clone(),
				Loc(Id::Rdfs(Rdfs::Comment), loc.clone()),
				Loc(
					Object::Literal(Literal::String(Loc(long.into(), long_loc.clone()))),
					long_loc,
				),
				None,
			),
			loc.clone(),
		))
	}

	if !short.is_empty() {
		quads.push(Loc(
			Quad(
				subject,
				Loc(Id::Rdfs(Rdfs::Comment), loc.clone()),
				Loc(
					Object::Literal(Literal::String(Loc(short.into(), short_loc.clone()))),
					short_loc,
				),
				None,
			),
			loc,
		))
	}
}

impl<F: Clone> Build<F> for Loc<crate::TypeDefinition<F>, F> {
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
				Loc(Subject::Iri(id), id_loc.clone()),
				Loc(Id::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Id::Rdfs(Rdfs::Class)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Subject::Iri(id), id_loc.clone()), quads)
		}

		for property in def.properties.into_value() {
			ctx.scope = Some(id);
			let Loc(prop, prop_loc) = property.build(ctx, quads)?;
			ctx.scope = None;

			quads.push(Loc(
				Quad(
					Loc(Subject::Iri(prop), prop_loc.clone()),
					Loc(Id::Rdfs(Rdfs::Domain), prop_loc.clone()),
					Loc(Object::Iri(id), id_loc.clone()),
					None,
				),
				prop_loc,
			));
		}

		implicit_layout.build(ctx, quads)
	}
}

impl<F: Clone> Build<F> for Loc<crate::PropertyDefinition<F>, F> {
	type Target = Loc<Id, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(def, loc) = self;
		let Loc(id, id_loc) = def.id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Subject::Iri(id), id_loc.clone()),
				Loc(Id::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Id::Rdf(Rdf::Property)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Subject::Iri(id), id_loc.clone()), quads)
		}

		if let Some(Loc(ty, _)) = def.ty {
			let scope = ctx.scope.take();
			let object = ty.expr.build(ctx, quads)?;
			ctx.scope = scope;

			quads.push(Loc(
				Quad(
					Loc(Subject::Iri(id), id_loc.clone()),
					Loc(Id::Rdfs(Rdfs::Range), object.location().clone()),
					object,
					None,
				),
				loc,
			));

			for Loc(ann, ann_loc) in ty.annotations {
				match ann {
					crate::Annotation::Multiple => quads.push(Loc(
						Quad(
							Loc(Subject::Iri(id), id_loc.clone()),
							Loc(Id::Schema(Schema::MultipleValues), ann_loc.clone()),
							Loc(Object::Iri(Id::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
					crate::Annotation::Required => quads.push(Loc(
						Quad(
							Loc(Subject::Iri(id), id_loc.clone()),
							Loc(Id::Schema(Schema::ValueRequired), ann_loc.clone()),
							Loc(Object::Iri(Id::Schema(Schema::True)), ann_loc.clone()),
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

impl<F: Clone> Build<F> for Loc<crate::TypeExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::TypeExpr::Id(id) => {
				let Loc(id, _) = id.build(ctx, quads)?;
				Ok(Loc(Object::Iri(id), loc))
			}
			crate::TypeExpr::Reference(r) => r.build(ctx, quads),
		}
	}
}

impl<F: Clone> Build<F> for Loc<crate::LayoutDefinition<F>, F> {
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
				Loc(Subject::Iri(id), id_loc.clone()),
				Loc(Id::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Id::TreeLdr(TreeLdr::Layout)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		let for_loc = id_loc.clone().with(ty_id_loc.span());
		quads.push(Loc(
			Quad(
				Loc(Subject::Iri(id), id_loc.clone()),
				Loc(Id::TreeLdr(TreeLdr::LayoutFor), id_loc.clone()),
				Loc(Object::Iri(ty_id), ty_id_loc),
				None,
			),
			for_loc,
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Subject::Iri(id), id_loc.clone()), quads)
		}

		let Loc(fields, fields_loc) = def.fields;
		let mut fields_head = Loc(Object::Iri(Id::Rdf(Rdf::Nil)), fields_loc);
		for field in fields.into_iter().rev() {
			ctx.scope = Some(ty_id);

			let item_label = ctx.namespace.new_blank_label();
			let first = field.build(ctx, quads)?;
			let first_loc = first.location().clone();

			quads.push(Loc(
				Quad(
					Loc(Subject::Blank(item_label), first_loc.clone()),
					Loc(Id::Rdf(Rdf::First), first_loc.clone()),
					first,
					None,
				),
				first_loc.clone(),
			));

			quads.push(Loc(
				Quad(
					Loc(Subject::Blank(item_label), first_loc.clone()),
					Loc(Id::Rdf(Rdf::Rest), first_loc.clone()),
					fields_head,
					None,
				),
				first_loc.clone(),
			));

			fields_head = Loc(Object::Blank(item_label), first_loc);

			ctx.scope = None;
		}

		quads.push(Loc(
			Quad(
				Loc(Subject::Iri(id), id_loc.clone()),
				Loc(Id::TreeLdr(TreeLdr::Fields), id_loc.clone()),
				fields_head,
				None,
			),
			id_loc,
		));

		Ok(())
	}
}

impl<F: Clone> Build<F> for Loc<crate::FieldDefinition<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(def, loc) = self;

		let label = ctx.namespace.new_blank_label();
		let Loc(prop_id, prop_id_loc) = def.id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Subject::Blank(label), loc.clone()),
				Loc(Id::Rdf(Rdf::Type), loc.clone()),
				Loc(Object::Iri(Id::TreeLdr(TreeLdr::Field)), loc.clone()),
				None,
			),
			loc.clone(),
		));

		quads.push(Loc(
			Quad(
				Loc(Subject::Blank(label), prop_id_loc.clone()),
				Loc(Id::TreeLdr(TreeLdr::FieldFor), prop_id_loc.clone()),
				Loc(Object::Iri(prop_id), prop_id_loc.clone()),
				None,
			),
			prop_id_loc.clone(),
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Subject::Blank(label), prop_id_loc.clone()), quads)
		}

		let Loc(name, name_loc) = match def.alias {
			Some(Loc(alias, alias_loc)) => Loc(alias.into_string(), alias_loc),
			None => Loc(
				prop_id
					.iri(&ctx.namespace)
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
				Loc(Subject::Blank(label), prop_id_loc.clone()),
				Loc(Id::Rdfs(Rdfs::Label), prop_id_loc.clone()),
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
			ctx.scope = scope;
			quads.push(Loc(
				Quad(
					Loc(Subject::Blank(label), prop_id_loc.clone()),
					Loc(Id::Rdfs(Rdfs::Range), object.location().clone()),
					object,
					None,
				),
				loc.clone(),
			));

			for Loc(ann, ann_loc) in layout.annotations {
				match ann {
					crate::Annotation::Multiple => quads.push(Loc(
						Quad(
							Loc(Subject::Blank(label), prop_id_loc.clone()),
							Loc(Id::Schema(Schema::MultipleValues), ann_loc.clone()),
							Loc(Object::Iri(Id::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
					crate::Annotation::Required => quads.push(Loc(
						Quad(
							Loc(Subject::Blank(label), prop_id_loc.clone()),
							Loc(Id::Schema(Schema::ValueRequired), ann_loc.clone()),
							Loc(Object::Iri(Id::Schema(Schema::True)), ann_loc.clone()),
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

impl<F: Clone> Build<F> for Loc<crate::LayoutExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::LayoutExpr::Id(id) => {
				let Loc(id, _) = id.build(ctx, quads)?;
				Ok(Loc(Object::Iri(id), loc))
			}
			crate::LayoutExpr::Reference(r) => {
				let deref_ty = r.build(ctx, quads)?;
				let ty = ctx.namespace.new_blank_label();
				quads.push(Loc(
					Quad(
						Loc(Subject::Blank(ty), loc.clone()),
						Loc(Id::Rdf(Rdf::Type), loc.clone()),
						Loc(Object::Iri(Id::TreeLdr(TreeLdr::Reference)), loc.clone()),
						None,
					),
					loc.clone(),
				));
				quads.push(Loc(
					Quad(
						Loc(Subject::Blank(ty), loc.clone()),
						Loc(Id::TreeLdr(TreeLdr::DerefTo), loc.clone()),
						deref_ty,
						None,
					),
					loc.clone(),
				));
				Ok(Loc(Object::Blank(ty), loc))
			}
		}
	}
}
