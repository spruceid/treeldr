use iref::{Iri, IriBuf};
use iref_enum::IriEnum;
use locspan::Loc;
use rdf_types::{loc::Literal, Quad};
use std::{collections::HashMap, fmt};

mod display;
mod name;

pub use display::*;
pub use name::*;

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

	#[iri("tldr:name")]
	Name,

	#[iri("tldr:fieldFor")]
	FieldFor,

	#[iri("tldr:derefTo")]
	DerefTo,

	/// Layout equality constraint.
	///
	/// The only possible instance of the subject layout is the given object.
	#[iri("tldr:singleton")]
	Singleton,

	/// Layout regular expression matching constraint.
	///
	/// The instances of the subject layout must match the given regular
	/// expression object.
	#[iri("tldr:matches")]
	Matches,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[iri_prefix("owl" = "http://www.w3.org/2002/07/owl#")]
pub enum Owl {
	#[iri("owl:unionOf")]
	UnionOf
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

/// UnknownTerm index.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct UnknownTerm(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Term {
	Rdf(Rdf),
	Rdfs(Rdfs),
	Schema(Schema),
	Owl(Owl),
	TreeLdr(TreeLdr),
	Unknown(UnknownTerm),
}

impl Term {
	pub fn try_from_iri(iri: Iri, ns: &Vocabulary) -> Option<Self> {
		match Rdf::try_from(iri) {
			Ok(id) => Some(Term::Rdf(id)),
			Err(_) => match Rdfs::try_from(iri) {
				Ok(id) => Some(Term::Rdfs(id)),
				Err(_) => match Schema::try_from(iri) {
					Ok(id) => Some(Term::Schema(id)),
					Err(_) => match TreeLdr::try_from(iri) {
						Ok(id) => Some(Term::TreeLdr(id)),
						Err(_) => {
							let iri_buf: IriBuf = iri.into();
							ns.get(&iri_buf).map(Term::Unknown)
						}
					},
				},
			},
		}
	}

	pub fn from_iri(iri: IriBuf, ns: &mut Vocabulary) -> Self {
		match Rdf::try_from(iri.as_iri()) {
			Ok(id) => Term::Rdf(id),
			Err(_) => match Rdfs::try_from(iri.as_iri()) {
				Ok(id) => Term::Rdfs(id),
				Err(_) => match Schema::try_from(iri.as_iri()) {
					Ok(id) => Term::Schema(id),
					Err(_) => match Owl::try_from(iri.as_iri()) {
						Ok(id) => Term::Owl(id),
						Err(_) => match TreeLdr::try_from(iri.as_iri()) {
							Ok(id) => Term::TreeLdr(id),
							Err(_) => Term::Unknown(ns.insert(iri)),
						},
					},
				},
			},
		}
	}

	pub fn iri<'n>(&self, ns: &'n Vocabulary) -> Option<Iri<'n>> {
		match self {
			Self::Rdf(id) => Some(id.into()),
			Self::Rdfs(id) => Some(id.into()),
			Self::Schema(id) => Some(id.into()),
			Self::Owl(id) => Some(id.into()),
			Self::TreeLdr(id) => Some(id.into()),
			Self::Unknown(name) => ns.iri(*name),
		}
	}
}

impl rdf_types::AsTerm for Term {
	type Iri = Self;
	type BlankId = BlankLabel;
	type Literal = rdf_types::Literal;

	fn as_term(&self) -> rdf_types::Term<&Self::Iri, &Self::BlankId, &Self::Literal> {
		rdf_types::Term::Iri(self)
	}
}

impl rdf_types::IntoTerm for Term {
	type Iri = Self;
	type BlankId = BlankLabel;
	type Literal = rdf_types::Literal;

	fn into_term(self) -> rdf_types::Term<Self::Iri, Self::BlankId, Self::Literal> {
		rdf_types::Term::Iri(self)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

pub type Id = rdf_types::Subject<Term, BlankLabel>;

pub type GraphLabel = rdf_types::GraphLabel<Term, BlankLabel>;

pub type Object<F> = rdf_types::Object<Term, BlankLabel, Literal<F>>;

pub type LocQuad<F> = rdf_types::loc::LocQuad<Id, Term, Object<F>, GraphLabel, F>;

pub type StrippedObject = rdf_types::Object<Term, BlankLabel, rdf_types::Literal>;

pub type StrippedQuad = rdf_types::Quad<Id, Term, StrippedObject, GraphLabel>;

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
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> Id {
	match subject {
		rdf_types::Subject::Iri(iri) => Id::Iri(Term::from_iri(iri, ns)),
		rdf_types::Subject::Blank(label) => Id::Blank(blank_label(label)),
	}
}

pub fn object_from_rdf<F>(
	object: rdf_types::loc::Object<F>,
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> Object<F> {
	match object {
		rdf_types::Object::Iri(iri) => Object::Iri(Term::from_iri(iri, ns)),
		rdf_types::Object::Blank(label) => Object::Blank(blank_label(label)),
		rdf_types::Object::Literal(lit) => Object::Literal(lit),
	}
}

pub fn stripped_object_from_rdf(
	object: rdf_types::Object,
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> StrippedObject {
	match object {
		rdf_types::Object::Iri(iri) => StrippedObject::Iri(Term::from_iri(iri, ns)),
		rdf_types::Object::Blank(label) => StrippedObject::Blank(blank_label(label)),
		rdf_types::Object::Literal(lit) => StrippedObject::Literal(lit),
	}
}

pub fn graph_label_from_rdf(
	graph_label: rdf_types::GraphLabel,
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> Id {
	match graph_label {
		rdf_types::GraphLabel::Iri(iri) => GraphLabel::Iri(Term::from_iri(iri, ns)),
		rdf_types::GraphLabel::Blank(label) => GraphLabel::Blank(blank_label(label)),
	}
}

pub fn loc_quad_from_rdf<F>(
	Loc(rdf_types::Quad(s, p, o, g), loc): rdf_types::loc::LocRdfQuad<F>,
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> LocQuad<F> {
	Loc(
		rdf_types::Quad(
			s.map(|s| subject_from_rdf(s, ns, &mut blank_label)),
			p.map(|p| Term::from_iri(p, ns)),
			o.map(|o| object_from_rdf(o, ns, &mut blank_label)),
			g.map(|g| g.map(|g| graph_label_from_rdf(g, ns, blank_label))),
		),
		loc,
	)
}

pub fn stripped_loc_quad_from_rdf<F>(
	Loc(rdf_types::Quad(s, p, o, g), _): rdf_types::loc::LocRdfQuad<F>,
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> StrippedQuad {
	use locspan::Strip;
	rdf_types::Quad(
		subject_from_rdf(s.into_value(), ns, &mut blank_label),
		Term::from_iri(p.into_value(), ns),
		stripped_object_from_rdf(o.strip(), ns, &mut blank_label),
		g.map(|g| graph_label_from_rdf(g.into_value(), ns, blank_label)),
	)
}

#[derive(Default)]
pub struct Vocabulary {
	map: Vec<IriBuf>,
	reverse: HashMap<IriBuf, UnknownTerm>,
	blank_label_count: u32,
}

impl Vocabulary {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, iri: &IriBuf) -> Option<UnknownTerm> {
		self.reverse.get(iri).cloned()
	}

	pub fn iri(&self, name: UnknownTerm) -> Option<Iri> {
		self.map.get(name.0).map(|iri| iri.as_iri())
	}

	pub fn new_blank_label(&mut self) -> BlankLabel {
		let label = BlankLabel(self.blank_label_count);
		self.blank_label_count += 1;
		label
	}

	pub fn insert(&mut self, iri: IriBuf) -> UnknownTerm {
		use std::collections::hash_map::Entry;
		match self.reverse.entry(iri) {
			Entry::Occupied(entry) => *entry.get(),
			Entry::Vacant(entry) => {
				let name = UnknownTerm(self.map.len());
				self.map.push(entry.key().clone());
				entry.insert(name);
				name
			}
		}
	}
}