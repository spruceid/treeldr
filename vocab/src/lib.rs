use iref::{Iri, IriBuf};
use iref_enum::IriEnum;
use locspan::Loc;
use rdf_types::{loc::Literal, Quad};
use std::{collections::HashMap, fmt};

mod display;

pub use display::*;

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

/// UnknownName index.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct UnknownName(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Name {
	Rdf(Rdf),
	Rdfs(Rdfs),
	Schema(Schema),
	TreeLdr(TreeLdr),
	Unknown(UnknownName),
}

impl Name {
	pub fn try_from_iri(iri: Iri, ns: &Vocabulary) -> Option<Name> {
		match Rdf::try_from(iri) {
			Ok(id) => Some(Name::Rdf(id)),
			Err(_) => match Rdfs::try_from(iri) {
				Ok(id) => Some(Name::Rdfs(id)),
				Err(_) => match Schema::try_from(iri) {
					Ok(id) => Some(Name::Schema(id)),
					Err(_) => match TreeLdr::try_from(iri) {
						Ok(id) => Some(Name::TreeLdr(id)),
						Err(_) => {
							let iri_buf: IriBuf = iri.into();
							ns.get(&iri_buf).map(Name::Unknown)
						}
					},
				},
			},
		}
	}

	pub fn from_iri(iri: IriBuf, ns: &mut Vocabulary) -> Name {
		match Rdf::try_from(iri.as_iri()) {
			Ok(id) => Name::Rdf(id),
			Err(_) => match Rdfs::try_from(iri.as_iri()) {
				Ok(id) => Name::Rdfs(id),
				Err(_) => match Schema::try_from(iri.as_iri()) {
					Ok(id) => Name::Schema(id),
					Err(_) => match TreeLdr::try_from(iri.as_iri()) {
						Ok(id) => Name::TreeLdr(id),
						Err(_) => Name::Unknown(ns.insert(iri)),
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
			Self::TreeLdr(id) => Some(id.into()),
			Self::Unknown(name) => ns.iri(*name),
		}
	}
}

impl rdf_types::AsTerm for Name {
	type Iri = Self;
	type BlankId = BlankLabel;
	type Literal = rdf_types::Literal;

	fn as_term(&self) -> rdf_types::Term<&Self::Iri, &Self::BlankId, &Self::Literal> {
		rdf_types::Term::Iri(self)
	}
}

impl rdf_types::IntoTerm for Name {
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

pub type Id = rdf_types::Subject<Name, BlankLabel>;

pub type GraphLabel = rdf_types::GraphLabel<Name, BlankLabel>;

pub type Object<F> = rdf_types::Object<Name, BlankLabel, Literal<F>>;

pub type LocQuad<F> = rdf_types::loc::LocQuad<Id, Name, Object<F>, GraphLabel, F>;

pub type StrippedObject = rdf_types::Object<Name, BlankLabel, rdf_types::Literal>;

pub type StrippedQuad = rdf_types::Quad<Id, Name, StrippedObject, GraphLabel>;

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
		rdf_types::Subject::Iri(iri) => Id::Iri(Name::from_iri(iri, ns)),
		rdf_types::Subject::Blank(label) => Id::Blank(blank_label(label)),
	}
}

pub fn object_from_rdf<F>(
	object: rdf_types::loc::Object<F>,
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> Object<F> {
	match object {
		rdf_types::Object::Iri(iri) => Object::Iri(Name::from_iri(iri, ns)),
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
		rdf_types::Object::Iri(iri) => StrippedObject::Iri(Name::from_iri(iri, ns)),
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
		rdf_types::GraphLabel::Iri(iri) => GraphLabel::Iri(Name::from_iri(iri, ns)),
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
			p.map(|p| Name::from_iri(p, ns)),
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
		Name::from_iri(p.into_value(), ns),
		stripped_object_from_rdf(o.strip(), ns, &mut blank_label),
		g.map(|g| graph_label_from_rdf(g.into_value(), ns, blank_label)),
	)
}

#[derive(Default)]
pub struct Vocabulary {
	map: Vec<IriBuf>,
	reverse: HashMap<IriBuf, UnknownName>,
	blank_label_count: u32,
}

impl Vocabulary {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, iri: &IriBuf) -> Option<UnknownName> {
		self.reverse.get(iri).cloned()
	}

	pub fn iri(&self, name: UnknownName) -> Option<Iri> {
		self.map.get(name.0).map(|iri| iri.as_iri())
	}

	pub fn new_blank_label(&mut self) -> BlankLabel {
		let label = BlankLabel(self.blank_label_count);
		self.blank_label_count += 1;
		label
	}

	pub fn insert(&mut self, iri: IriBuf) -> UnknownName {
		use std::collections::hash_map::Entry;
		match self.reverse.entry(iri) {
			Entry::Occupied(entry) => *entry.get(),
			Entry::Vacant(entry) => {
				let name = UnknownName(self.map.len());
				self.map.push(entry.key().clone());
				entry.insert(name);
				name
			}
		}
	}
}

// /// Unique identifier associated to a known IRI.
// ///
// /// This simplifies storage and comparison between IRIs.
// #[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
// pub struct Name(pub(crate) usize);

// impl Name {
// 	pub(crate) fn index(&self) -> usize {
// 		self.0
// 	}
// }

// /// Dictionary storing each known IRI and associated unique `Name`.
// #[derive(Default)]
// pub struct Vocabulary {
// 	iri_to_id: HashMap<IriBuf, Name>,
// 	id_to_iri: Vec<IriBuf>,
// }

// impl Vocabulary {
// 	/// Creates a new empty vocabulary.
// 	pub fn new() -> Self {
// 		Self::default()
// 	}

// 	/// Returns the `Name` of the given IRI, if any.
// 	pub fn id(&self, iri: &IriBuf) -> Option<Name> {
// 		self.iri_to_id.get(iri).cloned()
// 	}

// 	/// Returns the IRI of the given `Name`, if any.
// 	pub fn get(&self, id: Name) -> Option<Iri> {
// 		self.id_to_iri.get(id.index()).map(|iri| iri.as_iri())
// 	}

// 	/// Adds a new IRI to the vocabulary and returns its `Name`.
// 	///
// 	/// If the IRI is already in the vocabulary, its `Name` is returned
// 	/// and the vocabulary is unchanged.
// 	pub fn insert(&mut self, iri: IriBuf) -> Name {
// 		use std::collections::hash_map::Entry;
// 		match self.iri_to_id.entry(iri) {
// 			Entry::Occupied(entry) => *entry.get(),
// 			Entry::Vacant(entry) => {
// 				let id = Name(self.id_to_iri.len());
// 				self.id_to_iri.push(entry.key().clone());
// 				entry.insert(id);
// 				id
// 			}
// 		}
// 	}
// }
