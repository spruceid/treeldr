pub use crate::layout::Primitive;
use iref::{Iri, IriBuf};
use iref_enum::IriEnum;
use locspan::{Loc, Location, Meta, Span};
use rdf_types::Quad;
use std::{collections::HashMap, fmt};

mod display;

pub use display::*;

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[iri_prefix("tldr" = "https://treeldr.org/")]
pub enum TreeLdr {
	/// Property that reflects a resource.
	///
	/// Useful to capture the identifier of a resource in its layout.
	#[iri("tldr:self")]
	Self_,

	#[iri("tldr:Layout")]
	Layout,

	#[iri("tldr:layoutFor")]
	LayoutFor,

	/// Gives the layout of a field or enumeration variant.
	#[iri("tldr:format")]
	Format,

	/// Primitive layout.
	Primitive(Primitive),

	/// Derived primitive layout.
	#[iri("tldr:derivedFrom")]
	DerivedFrom,

	/// Primitive layout restrictions definition.
	#[iri("tldr:withRestrictions")]
	WithRestrictions,

	/// Layout alias.
	#[iri("tldr:alias")]
	Alias,

	/// Structure layout.
	#[iri("tldr:fields")]
	Fields,

	/// Structure layout field.
	///
	/// The name of the field (required) is given by the `treeldr:name`
	/// property.
	/// The payload of the variant (required) is given by the `treeldr:format`
	/// property.
	#[iri("tldr:Field")]
	Field,

	#[iri("tldr:name")]
	Name,

	#[iri("tldr:fieldFor")]
	FieldFor,

	/// Reference layout target.
	///
	/// Used to declare that a layout is a reference.
	/// The actual layout is an IRI-like layout representing the identifier of
	/// the referenced node, given as object of the property.
	#[iri("tldr:reference")]
	Reference,

	/// Enumeration layout.
	///
	/// Declares that a layout is an enumeration, and what list defined the
	/// items of the enumeration. List object must be a list of layouts.
	#[iri("tldr:enumeration")]
	Enumeration,

	/// Enumeration layout variant.
	///
	/// The name of the variant (required) is given by the `treeldr:name`
	/// property.
	/// The payload of the variant (optional) is given by the `treeldr:format`
	/// property.
	#[iri("tldr:Variant")]
	Variant,

	/// Required layout.
	///
	/// This is a simple container layout that contains a single instance of
	/// the given type.
	#[iri("tldr:required")]
	Required,

	/// Option layout.
	#[iri("tldr:option")]
	Option,

	/// Array layout.
	#[iri("tldr:array")]
	Array,

	/// Defines the `first` property for the list semantics of an array layout.
	#[iri("tldr:arrayListFirst")]
	ArrayListFirst,

	/// Defines the `rest` property for the list semantics of an array layout.
	#[iri("tldr:arrayListRest")]
	ArrayListRest,

	/// Defines the `nil` value for the list semantics of an array layout.
	#[iri("tldr:arrayListNil")]
	ArrayListNil,

	/// Set layout.
	#[iri("tldr:set")]
	Set,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[iri_prefix("owl" = "http://www.w3.org/2002/07/owl#")]
pub enum Owl {
	#[iri("owl:real")]
	Real,

	#[iri("owl:rational")]
	Rational,

	#[iri("owl:unionOf")]
	UnionOf,

	#[iri("owl:intersectionOf")]
	IntersectionOf,

	#[iri("owl:Restriction")]
	Restriction,

	#[iri("owl:onProperty")]
	OnProperty,

	#[iri("owl:allValuesFrom")]
	AllValuesFrom,

	#[iri("owl:someValuesFrom")]
	SomeValuesFrom,

	#[iri("owl:maxCardinality")]
	MaxCardinality,

	#[iri("owl:minCardinality")]
	MinCardinality,

	#[iri("owl:maxCardinality")]
	Cardinality,

	#[iri("owl:onDatatype")]
	OnDatatype,

	#[iri("owl:withRestrictions")]
	WithRestrictions,
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[iri_prefix("xsd" = "http://www.w3.org/2001/XMLSchema#")]
pub enum Xsd {
	#[iri("xsd:boolean")]
	Boolean,

	#[iri("xsd:decimal")]
	Decimal,

	#[iri("xsd:integer")]
	Integer,

	#[iri("xsd:nonNegativeInteger")]
	NonNegativeInteger,

	#[iri("xsd:positiveInteger")]
	PositiveInteger,

	#[iri("xsd:int")]
	Int,

	#[iri("xsd:float")]
	Float,

	#[iri("xsd:double")]
	Double,

	#[iri("xsd:string")]
	String,

	#[iri("xsd:time")]
	Time,

	#[iri("xsd:date")]
	Date,

	#[iri("xsd:dateTime")]
	DateTime,

	#[iri("xsd:duration")]
	Duration,

	#[iri("xsd:anyURI")]
	AnyUri,

	#[iri("xsd:length")]
	Length,

	#[iri("xsd:minLength")]
	MinLength,

	#[iri("xsd:maxLength")]
	MaxLength,

	#[iri("xsd:pattern")]
	Pattern,

	#[iri("xsd:maxExclusive")]
	MaxExclusive,

	#[iri("xsd:maxInclusive")]
	MaxInclusive,

	#[iri("xsd:minExclusive")]
	MinExclusive,

	#[iri("xsd:minInclusive")]
	MinInclusive,
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
	#[iri("rdfs:Resource")]
	Resource,

	#[iri("rdfs:Class")]
	Class,

	#[iri("rdfs:Datatype")]
	Datatype,

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
	Xsd(Xsd),
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
				Err(_) => match Xsd::try_from(iri) {
					Ok(id) => Some(Term::Xsd(id)),
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
			},
		}
	}

	pub fn from_iri(iri: IriBuf, ns: &mut Vocabulary) -> Self {
		match Rdf::try_from(iri.as_iri()) {
			Ok(id) => Term::Rdf(id),
			Err(_) => match Rdfs::try_from(iri.as_iri()) {
				Ok(id) => Term::Rdfs(id),
				Err(_) => match Xsd::try_from(iri.as_iri()) {
					Ok(id) => Term::Xsd(id),
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
			},
		}
	}

	pub fn iri<'n>(&self, ns: &'n Vocabulary) -> Option<Iri<'n>> {
		match self {
			Self::Rdf(id) => Some(id.into()),
			Self::Rdfs(id) => Some(id.into()),
			Self::Xsd(id) => Some(id.into()),
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
	type Literal = StrippedLiteral;

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

pub type Literal<F, N = Span> =
	rdf_types::meta::Literal<Location<F, N>, rdf_types::StringLiteral, Term>;

pub type Id = rdf_types::Subject<Term, BlankLabel>;

pub type GraphLabel = rdf_types::GraphLabel<Term, BlankLabel>;

pub type Object<F, N = Span> = rdf_types::Object<Term, BlankLabel, Literal<F, N>>;

pub type LocQuad<F, N = Span> =
	rdf_types::meta::MetaQuad<Id, Term, Object<F, N>, GraphLabel, Location<F, N>>;

pub type StrippedLiteral = rdf_types::Literal<rdf_types::StringLiteral, Term>;

pub type StrippedObject = rdf_types::Object<Term, BlankLabel, StrippedLiteral>;

pub type StrippedQuad = rdf_types::Quad<Id, Term, StrippedObject, GraphLabel>;

pub fn strip_quad<F>(Meta(rdf_types::Quad(s, p, o, g), _): LocQuad<F>) -> StrippedQuad {
	use locspan::Strip;
	rdf_types::Quad(
		s.into_value(),
		p.into_value(),
		o.strip(),
		g.map(|g| g.into_value()),
	)
}

pub fn literal_from_rdf<F>(
	literal: rdf_types::meta::Literal<Location<F>>,
	ns: &mut Vocabulary,
) -> Literal<F> {
	match literal {
		rdf_types::meta::Literal::String(s) => Literal::String(s),
		rdf_types::meta::Literal::LangString(s, tag) => Literal::LangString(s, tag),
		rdf_types::meta::Literal::TypedString(s, Meta(ty, ty_loc)) => {
			Literal::TypedString(s, Loc(Term::from_iri(ty, ns), ty_loc))
		}
	}
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
	object: rdf_types::meta::Object<Location<F>>,
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> Object<F> {
	match object {
		rdf_types::Object::Iri(iri) => Object::Iri(Term::from_iri(iri, ns)),
		rdf_types::Object::Blank(label) => Object::Blank(blank_label(label)),
		rdf_types::Object::Literal(lit) => Object::Literal(literal_from_rdf(lit, ns)),
	}
}

pub fn stripped_literal_from_rdf(
	literal: rdf_types::Literal,
	ns: &mut Vocabulary,
) -> StrippedLiteral {
	match literal {
		rdf_types::Literal::String(s) => StrippedLiteral::String(s),
		rdf_types::Literal::LangString(s, tag) => StrippedLiteral::LangString(s, tag),
		rdf_types::Literal::TypedString(s, ty) => {
			StrippedLiteral::TypedString(s, Term::from_iri(ty, ns))
		}
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
		rdf_types::Object::Literal(lit) => {
			StrippedObject::Literal(stripped_literal_from_rdf(lit, ns))
		}
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
	Meta(rdf_types::Quad(s, p, o, g), loc): rdf_types::meta::MetaRdfQuad<Location<F>>,
	ns: &mut Vocabulary,
	mut blank_label: impl FnMut(rdf_types::BlankIdBuf) -> BlankLabel,
) -> LocQuad<F> {
	Meta(
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
	Meta(rdf_types::Quad(s, p, o, g), _): rdf_types::meta::MetaRdfQuad<Location<F>>,
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

pub trait BorrowWithVocabulary {
	fn with_vocabulary<'v>(&self, vocabulary: &'v Vocabulary) -> WithVocabulary<'_, 'v, Self> {
		WithVocabulary(self, vocabulary)
	}
}

impl<T> BorrowWithVocabulary for T {}

pub struct WithVocabulary<'t, 'v, T: ?Sized>(&'t T, &'v Vocabulary);

impl<'t, 'v, T: ?Sized> WithVocabulary<'t, 'v, T> {
	pub fn value(&self) -> &'t T {
		self.0
	}

	pub fn vocabulary(&self) -> &'v Vocabulary {
		self.1
	}
}
