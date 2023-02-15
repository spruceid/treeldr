use crate::{
	prop::{PropertyName, UnknownProperty},
	vocab::{self, Xsd},
	Id, IriIndex, TId,
};

pub mod double;
pub mod float;
pub mod real;
pub mod string;

pub enum Restriction<'a> {
	Real(real::Restriction<'a>),
	Float(float::Restriction),
	Double(double::Restriction),
	String(string::Restriction<'a>),
}

#[derive(Clone, Copy)]
pub enum Restrictions<'a> {
	Real(&'a real::Restrictions),
	Float(&'a float::Restrictions),
	Double(&'a double::Restrictions),
	String(&'a string::Restrictions),
}

impl<'a> Restrictions<'a> {
	pub fn iter(&self) -> RestrictionsIter<'a> {
		match self {
			Self::Real(r) => RestrictionsIter::Real(r.iter()),
			Self::Float(r) => RestrictionsIter::Float(r.iter()),
			Self::Double(r) => RestrictionsIter::Double(r.iter()),
			Self::String(r) => RestrictionsIter::String(r.iter()),
		}
	}
}

pub enum RestrictionsIter<'a> {
	Real(real::Iter<'a>),
	Float(float::Iter),
	Double(double::Iter),
	String(string::Iter<'a>),
}

impl<'a> Iterator for RestrictionsIter<'a> {
	type Item = Restriction<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Real(r) => r.next().map(Restriction::Real),
			Self::Float(r) => r.next().map(Restriction::Float),
			Self::Double(r) => r.next().map(Restriction::Double),
			Self::String(r) => r.next().map(Restriction::String),
		}
	}
}

impl<'a> DoubleEndedIterator for RestrictionsIter<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::Real(r) => r.next_back().map(Restriction::Real),
			Self::Float(r) => r.next_back().map(Restriction::Float),
			Self::Double(r) => r.next_back().map(Restriction::Double),
			Self::String(r) => r.next_back().map(Restriction::String),
		}
	}
}

// #[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
// pub enum Numeric<T> {
// 	MinInclusive(T),
// 	MinExclusive(T),
// 	MaxInclusive(T),
// 	MaxExclusive(T),
// }

// impl<T> Numeric<T> {
// 	pub fn as_binding(&self) -> NumericBindingRef<T> {
// 		match self {
// 			Self::MinInclusive(v) => NumericBindingRef::MinInclusive(v),
// 			Self::MinExclusive(v) => NumericBindingRef::MinExclusive(v),
// 			Self::MaxInclusive(v) => NumericBindingRef::MaxInclusive(v),
// 			Self::MaxExclusive(v) => NumericBindingRef::MaxExclusive(v)
// 		}
// 	}
// }

// #[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
// pub enum String {
// 	MinLength(value::Integer),
// 	MaxLength(value::Integer),
// 	Pattern(RegExp),
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	MinInclusive(Option<TId<UnknownProperty>>),
	MinExclusive(Option<TId<UnknownProperty>>),
	MaxInclusive(Option<TId<UnknownProperty>>),
	MaxExclusive(Option<TId<UnknownProperty>>),
	MinLength(Option<TId<UnknownProperty>>),
	MaxLength(Option<TId<UnknownProperty>>),
	Pattern(Option<TId<UnknownProperty>>),
}

impl Property {
	pub fn id(&self) -> Id {
		match self {
			Self::MinInclusive(None) => Id::Iri(IriIndex::Iri(vocab::Term::Xsd(Xsd::MinInclusive))),
			Self::MinInclusive(Some(p)) => p.id(),
			Self::MinExclusive(None) => Id::Iri(IriIndex::Iri(vocab::Term::Xsd(Xsd::MinExclusive))),
			Self::MinExclusive(Some(p)) => p.id(),
			Self::MaxInclusive(None) => Id::Iri(IriIndex::Iri(vocab::Term::Xsd(Xsd::MaxInclusive))),
			Self::MaxInclusive(Some(p)) => p.id(),
			Self::MaxExclusive(None) => Id::Iri(IriIndex::Iri(vocab::Term::Xsd(Xsd::MaxExclusive))),
			Self::MaxExclusive(Some(p)) => p.id(),
			Self::MinLength(None) => Id::Iri(IriIndex::Iri(vocab::Term::Xsd(Xsd::MinLength))),
			Self::MinLength(Some(p)) => p.id(),
			Self::MaxLength(None) => Id::Iri(IriIndex::Iri(vocab::Term::Xsd(Xsd::MaxLength))),
			Self::MaxLength(Some(p)) => p.id(),
			Self::Pattern(None) => Id::Iri(IriIndex::Iri(vocab::Term::Xsd(Xsd::Pattern))),
			Self::Pattern(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		match self {
			Self::MinInclusive(None) => Some(vocab::Term::Xsd(Xsd::MinInclusive)),
			Self::MinExclusive(None) => Some(vocab::Term::Xsd(Xsd::MinExclusive)),
			Self::MaxInclusive(None) => Some(vocab::Term::Xsd(Xsd::MaxInclusive)),
			Self::MaxExclusive(None) => Some(vocab::Term::Xsd(Xsd::MaxExclusive)),
			Self::MinLength(None) => Some(vocab::Term::Xsd(Xsd::MinLength)),
			Self::MaxLength(None) => Some(vocab::Term::Xsd(Xsd::MaxLength)),
			Self::Pattern(None) => Some(vocab::Term::Xsd(Xsd::Pattern)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::MinInclusive(None) => PropertyName::Resource("inclusive minimum"),
			Self::MinInclusive(Some(p)) => PropertyName::Other(*p),
			Self::MinExclusive(None) => PropertyName::Resource("exclusive minimum"),
			Self::MinExclusive(Some(p)) => PropertyName::Other(*p),
			Self::MaxInclusive(None) => PropertyName::Resource("inclusive maximum"),
			Self::MaxInclusive(Some(p)) => PropertyName::Other(*p),
			Self::MaxExclusive(None) => PropertyName::Resource("exclusive maximum"),
			Self::MaxExclusive(Some(p)) => PropertyName::Other(*p),
			Self::MinLength(None) => PropertyName::Resource("minimum length"),
			Self::MinLength(Some(p)) => PropertyName::Other(*p),
			Self::MaxLength(None) => PropertyName::Resource("maximum length"),
			Self::MaxLength(Some(p)) => PropertyName::Other(*p),
			Self::Pattern(None) => PropertyName::Resource("pattern"),
			Self::Pattern(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_type(&self) -> bool {
		false
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}
