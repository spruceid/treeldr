use crate::{node::BindingValueRef, vocab, Id, IriIndex, TId};

pub mod regexp;
pub mod restriction;

use derivative::Derivative;
use locspan::Meta;
pub use regexp::RegExp;
pub use restriction::{Restriction, Restrictions};

#[derive(Debug, Clone)]
pub enum DataType<M> {
	Primitive(Primitive),
	Derived(Derived<M>),
}

impl<M> DataType<M> {
	pub fn primitive(&self) -> Primitive {
		match self {
			Self::Primitive(p) => *p,
			Self::Derived(d) => d.primitive(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Definition<M> {
	desc: DataType<M>,
}

impl<M> Definition<M> {
	pub fn new(desc: DataType<M>) -> Self {
		Self { desc }
	}

	pub fn description(&self) -> &DataType<M> {
		&self.desc
	}

	pub fn on_datatype(&self) -> Option<&Meta<TId<crate::ty::DataType<M>>, M>> {
		match &self.desc {
			DataType::Derived(d) => Some(d.base()),
			_ => None,
		}
	}

	pub fn with_restrictions(&self) -> Option<Meta<Restrictions, &M>> {
		match &self.desc {
			DataType::Derived(d) => d.restrictions(),
			_ => None,
		}
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			on_datatype: self.on_datatype(),
			with_restrictions: self.with_restrictions(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
	/// `xsd:boolean`.
	Boolean,

	/// `owl:real`.
	Real,

	/// `xsd:float`.
	Float,

	/// `xsd:double`.
	Double,

	/// `xsd:string`.
	String,

	/// `xsd:date`.
	Date,

	/// `xsd:time`.
	Time,

	/// `xsd:dateTime`.
	DateTime,

	/// `xsd:duration`.
	Duration,
}

impl Primitive {
	pub fn id(&self) -> Id {
		use vocab::{Owl, Term, Xsd};
		match self {
			Self::Boolean => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Boolean))),
			Self::Real => Id::Iri(IriIndex::Iri(Term::Owl(Owl::Real))),
			Self::Float => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Float))),
			Self::Double => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Double))),
			Self::String => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::String))),
			Self::Date => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Date))),
			Self::Time => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Time))),
			Self::DateTime => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::DateTime))),
			Self::Duration => Id::Iri(IriIndex::Iri(Term::Xsd(Xsd::Duration))),
		}
	}

	pub fn from_iri(iri: IriIndex) -> Option<Self> {
		use vocab::{Owl, Term, Xsd};
		match iri {
			IriIndex::Iri(Term::Xsd(Xsd::Boolean)) => Some(Self::Boolean),
			IriIndex::Iri(Term::Owl(Owl::Real)) => Some(Self::Real),
			IriIndex::Iri(Term::Xsd(Xsd::Float)) => Some(Self::Float),
			IriIndex::Iri(Term::Xsd(Xsd::Double)) => Some(Self::Double),
			IriIndex::Iri(Term::Xsd(Xsd::String)) => Some(Self::String),
			IriIndex::Iri(Term::Xsd(Xsd::Date)) => Some(Self::Date),
			IriIndex::Iri(Term::Xsd(Xsd::Time)) => Some(Self::Time),
			IriIndex::Iri(Term::Xsd(Xsd::DateTime)) => Some(Self::DateTime),
			IriIndex::Iri(Term::Xsd(Xsd::Duration)) => Some(Self::Duration),
			_ => None,
		}
	}

	pub fn from_id(id: Id) -> Option<Self> {
		match id {
			Id::Iri(iri) => Self::from_iri(iri),
			Id::Blank(_) => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum Derived<M> {
	Boolean(Meta<TId<crate::ty::DataType<M>>, M>),
	Real(
		Meta<TId<crate::ty::DataType<M>>, M>,
		Meta<restriction::real::Restrictions, M>,
	),
	Float(
		Meta<TId<crate::ty::DataType<M>>, M>,
		Meta<restriction::float::Restrictions, M>,
	),
	Double(
		Meta<TId<crate::ty::DataType<M>>, M>,
		Meta<restriction::double::Restrictions, M>,
	),
	String(
		Meta<TId<crate::ty::DataType<M>>, M>,
		Meta<restriction::string::Restrictions, M>,
	),
	Date(Meta<TId<crate::ty::DataType<M>>, M>),
	Time(Meta<TId<crate::ty::DataType<M>>, M>),
	DateTime(Meta<TId<crate::ty::DataType<M>>, M>),
	Duration(Meta<TId<crate::ty::DataType<M>>, M>),
}

impl<M> Derived<M> {
	pub fn base(&self) -> &Meta<TId<crate::ty::DataType<M>>, M> {
		match self {
			Self::Boolean(id) => id,
			Self::Real(id, _) => id,
			Self::Float(id, _) => id,
			Self::Double(id, _) => id,
			Self::String(id, _) => id,
			Self::Date(id) => id,
			Self::Time(id) => id,
			Self::DateTime(id) => id,
			Self::Duration(id) => id,
		}
	}

	pub fn primitive(&self) -> Primitive {
		match self {
			Self::Boolean(_) => Primitive::Boolean,
			Self::Real(_, _) => Primitive::Real,
			Self::Float(_, _) => Primitive::Float,
			Self::Double(_, _) => Primitive::Double,
			Self::String(_, _) => Primitive::String,
			Self::Date(_) => Primitive::Date,
			Self::Time(_) => Primitive::Time,
			Self::DateTime(_) => Primitive::DateTime,
			Self::Duration(_) => Primitive::Duration,
		}
	}

	pub fn restrictions(&self) -> Option<Meta<Restrictions, &M>> {
		match self {
			Self::Real(_, Meta(r, m)) => Some(Meta(Restrictions::Real(r), m)),
			Self::Float(_, Meta(r, m)) => Some(Meta(Restrictions::Float(r), m)),
			Self::Double(_, Meta(r, m)) => Some(Meta(Restrictions::Double(r), m)),
			Self::String(_, Meta(r, m)) => Some(Meta(Restrictions::String(r), m)),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	OnDatatype,
	WithRestrictions,
}

impl Property {
	pub fn term(&self) -> vocab::Term {
		use vocab::{Owl, Term};
		match self {
			Self::OnDatatype => Term::Owl(Owl::OnDatatype),
			Self::WithRestrictions => Term::Owl(Owl::WithRestrictions),
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::OnDatatype => "restricted datatype",
			Self::WithRestrictions => "datatype restrictions",
		}
	}

	pub fn expect_type(&self) -> bool {
		matches!(self, Self::OnDatatype)
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}

pub enum ClassBindingRef<'a, M> {
	OnDatatype(TId<crate::ty::DataType<M>>),
	WithRestrictions(Restrictions<'a>),
}

pub type BindingRef<'a, M> = ClassBindingRef<'a, M>;

impl<'a, M> ClassBindingRef<'a, M> {
	pub fn property(&self) -> Property {
		match self {
			Self::OnDatatype(_) => Property::OnDatatype,
			Self::WithRestrictions(_) => Property::WithRestrictions,
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::OnDatatype(v) => BindingValueRef::DataType(*v),
			Self::WithRestrictions(v) => BindingValueRef::DatatypeRestrictions(*v),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	on_datatype: Option<&'a Meta<TId<crate::ty::DataType<M>>, M>>,
	with_restrictions: Option<Meta<Restrictions<'a>, &'a M>>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_datatype
			.take()
			.map(|m| {
				m.borrow()
					.into_cloned_value()
					.map(ClassBindingRef::OnDatatype)
			})
			.or_else(|| {
				self.with_restrictions
					.take()
					.map(|m| m.map(ClassBindingRef::WithRestrictions))
			})
	}
}
