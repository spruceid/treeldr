use std::fmt;

pub use xsd_types::value::*;

use crate::{Id, IriIndex};

mod numeric;

pub use numeric::*;

/// Value.
pub enum Value {
	Node(Id),
	Literal(Literal),
}

/// Literal value.
pub enum Literal {
	String(String),
	Numeric(Numeric),
}

pub trait AsRdfLiteral: Sized + fmt::Display {
	fn rdf_type(&self) -> IriIndex;

	fn as_rdf_literal(&self) -> rdf_types::Literal<rdf_types::StringLiteral, IriIndex> {
		rdf_types::Literal::TypedString(self.to_string().into(), self.rdf_type())
	}
}

impl AsRdfLiteral for Real {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Owl(crate::vocab::Owl::Real))
	}
}

impl AsRdfLiteral for Rational {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Owl(crate::vocab::Owl::Real))
	}
}

impl AsRdfLiteral for Float {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Float))
	}
}

impl AsRdfLiteral for Double {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Double))
	}
}

impl AsRdfLiteral for Decimal {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Decimal))
	}
}

impl AsRdfLiteral for Integer {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::Integer))
	}
}

impl AsRdfLiteral for NonNegativeInteger {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(
			crate::vocab::Xsd::NonNegativeInteger,
		))
	}
}

// impl AsRdfLiteral for PositiveInteger {
// 	fn rdf_type(&self) -> IriIndex {
// 		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::PositiveInteger))
// 	}
// }

impl AsRdfLiteral for NonPositiveInteger {
	fn rdf_type(&self) -> IriIndex {
		IriIndex::Iri(crate::vocab::Term::Xsd(
			crate::vocab::Xsd::NonPositiveInteger,
		))
	}
}

// impl AsRdfLiteral for NegativeInteger {
// 	fn rdf_type(&self) -> IriIndex {
// 		IriIndex::Iri(crate::vocab::Term::Xsd(crate::vocab::Xsd::NegativeInteger))
// 	}
// }
