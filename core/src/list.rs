use crate::{
	prop::{PropertyName, UnknownProperty},
	vocab, Id, IriIndex, TId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	First(Option<TId<UnknownProperty>>),
	Rest(Option<TId<UnknownProperty>>),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::{Rdf, Term};
		match self {
			Self::First(None) => Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::First))),
			Self::First(Some(p)) => p.id(),
			Self::Rest(None) => Id::Iri(IriIndex::Iri(Term::Rdf(Rdf::Rest))),
			Self::Rest(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::{Rdf, Term};
		match self {
			Self::First(None) => Some(Term::Rdf(Rdf::First)),
			Self::Rest(None) => Some(Term::Rdf(Rdf::Rest)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::First(None) => PropertyName::Resource("first item"),
			Self::First(Some(p)) => PropertyName::Other(*p),
			Self::Rest(None) => PropertyName::Resource("rest"),
			Self::Rest(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_type(&self) -> bool {
		false
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}
