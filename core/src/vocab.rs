pub use crate::layout::Primitive;
use iref::Iri;
use iref_enum::IriEnum;
use locspan::Meta;
use rdf_types::IriVocabularyMut;

pub type BlankIdIndex = rdf_types::vocabulary::Index;

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

	/// "One or many" layout.
	#[iri("tldr:oneOrMany")]
	OneOrMany,

	#[iri("tldr:intersectionOf")]
	IntersectionOf
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

	#[iri("owl:FunctionalProperty")]
	FunctionalProperty
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

/// IRI index.
///
/// This can be used as an IRI identifier that mixes IRIs that are statically
/// known (of type `I`) and IRIs added at run time with a dynamic index.
///
/// This type can directly be used as an IRI identifier with the
/// `IndexVocabulary` type.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum IriIndex {
	/// Index of the IRI in the vocabulary.
	Index(usize),

	/// Non indexed IRI.
	Iri(Term),
}

impl From<usize> for IriIndex {
	fn from(i: usize) -> Self {
		Self::Index(i)
	}
}

impl<'a> TryFrom<Iri<'a>> for IriIndex {
	type Error = UnknownTerm;

	fn try_from(value: Iri<'a>) -> Result<Self, Self::Error> {
		Ok(Self::Iri(Term::try_from(value)?))
	}
}

impl<V: rdf_types::IriVocabulary<Iri = IriIndex>> contextual::DisplayWithContext<V> for IriIndex {
	fn fmt_with(&self, vocabulary: &V, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		std::fmt::Display::fmt(&vocabulary.iri(self).unwrap(), f)
	}
}

impl<V: rdf_types::IriVocabulary<Iri = IriIndex>> rdf_types::RdfDisplayWithContext<V> for IriIndex {
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "<{}>", &vocabulary.iri(self).unwrap())
	}
}

impl rdf_types::AsTerm for IriIndex {
	type Iri = Self;
	type BlankId = BlankIdIndex;
	type Literal = StrippedLiteral;

	fn as_term(&self) -> rdf_types::Term<&Self, &Self::BlankId, &Self::Literal> {
		rdf_types::Term::Iri(self)
	}
}

impl rdf_types::IntoTerm for IriIndex {
	type Iri = Self;
	type BlankId = BlankIdIndex;
	type Literal = rdf_types::Literal;

	fn into_term(self) -> rdf_types::Term<Self, Self::BlankId, Self::Literal> {
		rdf_types::Term::Iri(self)
	}
}

impl rdf_types::vocabulary::IndexedIri for IriIndex {
	fn index(&self) -> rdf_types::vocabulary::IriIndex<Iri<'_>> {
		match self {
			Self::Iri(i) => rdf_types::vocabulary::IriIndex::Iri(i.iri()),
			Self::Index(i) => rdf_types::vocabulary::IriIndex::Index(*i),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Term {
	Rdf(Rdf),
	Rdfs(Rdfs),
	Xsd(Xsd),
	Schema(Schema),
	Owl(Owl),
	TreeLdr(TreeLdr),
}

impl Term {
	pub fn try_from_iri(iri: Iri) -> Option<Self> {
		match Rdf::try_from(iri) {
			Ok(id) => Some(Term::Rdf(id)),
			Err(_) => match Rdfs::try_from(iri) {
				Ok(id) => Some(Term::Rdfs(id)),
				Err(_) => match Xsd::try_from(iri) {
					Ok(id) => Some(Term::Xsd(id)),
					Err(_) => match Schema::try_from(iri) {
						Ok(id) => Some(Term::Schema(id)),
						Err(_) => match Owl::try_from(iri) {
							Ok(id) => Some(Term::Owl(id)),
							Err(_) => match TreeLdr::try_from(iri) {
								Ok(id) => Some(Term::TreeLdr(id)),
								Err(_) => None,
							},
						},
					},
				},
			},
		}
	}

	pub fn iri<'n>(&self) -> iref::Iri<'n> {
		match self {
			Self::Rdf(id) => id.into(),
			Self::Rdfs(id) => id.into(),
			Self::Xsd(id) => id.into(),
			Self::Schema(id) => id.into(),
			Self::Owl(id) => id.into(),
			Self::TreeLdr(id) => id.into(),
		}
	}
}

impl iref::AsIri for Term {
	fn as_iri(&self) -> Iri {
		self.iri()
	}
}

pub struct UnknownTerm;

impl<'a> TryFrom<Iri<'a>> for Term {
	type Error = UnknownTerm;

	fn try_from(iri: Iri<'a>) -> Result<Self, Self::Error> {
		Self::try_from_iri(iri).ok_or(UnknownTerm)
	}
}

pub type Literal<M> = rdf_types::meta::Literal<M, rdf_types::StringLiteral, IriIndex>;

pub type Id = rdf_types::Subject<IriIndex, BlankIdIndex>;

pub type GraphLabel = rdf_types::GraphLabel<IriIndex, BlankIdIndex>;

pub type Object<M> = rdf_types::Object<IriIndex, BlankIdIndex, Literal<M>>;

pub type LocQuad<M> = rdf_types::meta::MetaQuad<Id, IriIndex, Object<M>, GraphLabel, M>;

pub type StrippedLiteral = rdf_types::Literal<rdf_types::StringLiteral, IriIndex>;

pub type StrippedObject = rdf_types::Object<IriIndex, BlankIdIndex, StrippedLiteral>;

pub type StrippedQuad = rdf_types::Quad<Id, IriIndex, StrippedObject, GraphLabel>;

pub fn strip_quad<M>(Meta(rdf_types::Quad(s, p, o, g), _): LocQuad<M>) -> StrippedQuad {
	use locspan::Strip;
	rdf_types::Quad(
		s.into_value(),
		p.into_value(),
		o.strip(),
		g.map(|g| g.into_value()),
	)
}

pub fn literal_from_rdf<M>(
	literal: rdf_types::meta::Literal<M>,
	ns: &mut impl IriVocabularyMut<Iri = IriIndex>,
) -> Literal<M> {
	match literal {
		rdf_types::meta::Literal::String(s) => Literal::String(s),
		rdf_types::meta::Literal::LangString(s, tag) => Literal::LangString(s, tag),
		rdf_types::meta::Literal::TypedString(s, Meta(ty, ty_loc)) => {
			Literal::TypedString(s, Meta(ns.insert(ty.as_iri()), ty_loc))
		}
	}
}

pub fn subject_from_rdf<V: IriVocabularyMut<Iri = IriIndex>>(
	subject: rdf_types::Subject,
	ns: &mut V,
	mut blank_label: impl FnMut(&mut V, rdf_types::BlankIdBuf) -> BlankIdIndex,
) -> Id {
	match subject {
		rdf_types::Subject::Iri(iri) => Id::Iri(ns.insert(iri.as_iri())),
		rdf_types::Subject::Blank(label) => Id::Blank(blank_label(ns, label)),
	}
}

pub fn object_from_rdf<M, V: IriVocabularyMut<Iri = IriIndex>>(
	object: rdf_types::meta::Object<M>,
	ns: &mut V,
	mut blank_label: impl FnMut(&mut V, rdf_types::BlankIdBuf) -> BlankIdIndex,
) -> Object<M> {
	match object {
		rdf_types::Object::Iri(iri) => Object::Iri(ns.insert(iri.as_iri())),
		rdf_types::Object::Blank(label) => Object::Blank(blank_label(ns, label)),
		rdf_types::Object::Literal(lit) => Object::Literal(literal_from_rdf(lit, ns)),
	}
}

pub fn stripped_literal_from_rdf(
	literal: rdf_types::Literal,
	ns: &mut impl IriVocabularyMut<Iri = IriIndex>,
) -> StrippedLiteral {
	match literal {
		rdf_types::Literal::String(s) => StrippedLiteral::String(s),
		rdf_types::Literal::LangString(s, tag) => StrippedLiteral::LangString(s, tag),
		rdf_types::Literal::TypedString(s, ty) => {
			StrippedLiteral::TypedString(s, ns.insert(ty.as_iri()))
		}
	}
}

pub fn stripped_object_from_rdf<V: IriVocabularyMut<Iri = IriIndex>>(
	object: rdf_types::Object,
	ns: &mut V,
	mut blank_label: impl FnMut(&mut V, rdf_types::BlankIdBuf) -> BlankIdIndex,
) -> StrippedObject {
	match object {
		rdf_types::Object::Iri(iri) => StrippedObject::Iri(ns.insert(iri.as_iri())),
		rdf_types::Object::Blank(label) => StrippedObject::Blank(blank_label(ns, label)),
		rdf_types::Object::Literal(lit) => {
			StrippedObject::Literal(stripped_literal_from_rdf(lit, ns))
		}
	}
}

pub fn graph_label_from_rdf<V: IriVocabularyMut<Iri = IriIndex>>(
	graph_label: rdf_types::GraphLabel,
	ns: &mut V,
	mut blank_label: impl FnMut(&mut V, rdf_types::BlankIdBuf) -> BlankIdIndex,
) -> Id {
	match graph_label {
		rdf_types::GraphLabel::Iri(iri) => GraphLabel::Iri(ns.insert(iri.as_iri())),
		rdf_types::GraphLabel::Blank(label) => GraphLabel::Blank(blank_label(ns, label)),
	}
}

pub fn loc_quad_from_rdf<M, V: IriVocabularyMut<Iri = IriIndex>>(
	Meta(rdf_types::Quad(s, p, o, g), loc): rdf_types::meta::MetaRdfQuad<M>,
	ns: &mut V,
	mut blank_label: impl FnMut(&mut V, rdf_types::BlankIdBuf) -> BlankIdIndex,
) -> LocQuad<M> {
	Meta(
		rdf_types::Quad(
			s.map(|s| subject_from_rdf(s, ns, &mut blank_label)),
			p.map(|p| ns.insert(p.as_iri())),
			o.map(|o| object_from_rdf(o, ns, &mut blank_label)),
			g.map(|g| g.map(|g| graph_label_from_rdf(g, ns, blank_label))),
		),
		loc,
	)
}

pub fn stripped_loc_quad_from_rdf<M, V: IriVocabularyMut<Iri = IriIndex>>(
	Meta(rdf_types::Quad(s, p, o, g), _): rdf_types::meta::MetaRdfQuad<M>,
	ns: &mut V,
	mut blank_label: impl FnMut(&mut V, rdf_types::BlankIdBuf) -> BlankIdIndex,
) -> StrippedQuad {
	use locspan::Strip;
	rdf_types::Quad(
		subject_from_rdf(s.into_value(), ns, &mut blank_label),
		ns.insert(p.as_iri()),
		stripped_object_from_rdf(o.strip(), ns, &mut blank_label),
		g.map(|g| graph_label_from_rdf(g.into_value(), ns, blank_label)),
	)
}
