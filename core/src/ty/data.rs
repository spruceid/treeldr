use crate::{
	node::BindingValueRef,
	prop::{PropertyName, UnknownProperty},
	property_values, vocab, Id, IriIndex, MetaOption, PropertyValue, PropertyValueRef,
	RequiredFunctionalPropertyValue, TId,
};

pub mod regexp;
pub mod restriction;

use derivative::Derivative;
use locspan::Meta;
pub use regexp::RegExp;
pub use restriction::{Restriction, Restrictions};

/// DataType.
#[derive(Debug, Clone)]
pub enum DataType<M> {
	Primitive(Option<Primitive>),
	Derived(RequiredFunctionalPropertyValue<Derived<M>, M>),
}

impl<M> DataType<M> {
	pub fn primitive(&self) -> Option<Primitive> {
		match self {
			Self::Primitive(p) => *p,
			Self::Derived(d) => Some(d.value().primitive()),
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

	pub fn on_datatype(&self) -> Option<OnDatatype<M>> {
		match &self.desc {
			DataType::Derived(d) => Some(OnDatatype(d)),
			_ => None,
		}
	}

	pub fn with_restrictions(&self) -> Option<WithRestrictions<M>> {
		match &self.desc {
			DataType::Derived(d) => d.value().with_restrictions(),
			_ => None,
		}
	}

	pub fn bindings(&self) -> Bindings<M> {
		ClassBindings {
			on_datatype: self.on_datatype().map(OnDatatype::into_iter),
			with_restrictions: self.with_restrictions().map(WithRestrictions::into_iter),
		}
	}
}

pub struct OnDatatype<'a, M>(&'a RequiredFunctionalPropertyValue<Derived<M>, M>);

impl<'a, M> OnDatatype<'a, M> {
	pub fn iter(&self) -> OnDatatypeIter<'a, M> {
		OnDatatypeIter(self.0.iter())
	}
}

impl<'a, M> IntoIterator for OnDatatype<'a, M> {
	type IntoIter = OnDatatypeIter<'a, M>;
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct OnDatatypeIter<'a, M>(property_values::required_functional::Iter<'a, Derived<M>, M>);

impl<'a, M> Iterator for OnDatatypeIter<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|v| {
			PropertyValueRef::new(v.sub_property, v.value.base().borrow())
				.into_cloned_class_binding(ClassBindingRef::OnDatatype)
		})
	}
}

pub struct WithRestrictions<'a, M> {
	value: Meta<Restrictions<'a>, &'a M>,
}

impl<'a, M> WithRestrictions<'a, M> {
	fn new(value: Meta<Restrictions<'a>, &'a M>) -> Self {
		Self { value }
	}
}

impl<'a, M> IntoIterator for WithRestrictions<'a, M> {
	type IntoIter = WithRestrictionsIter<'a, M>;
	type Item = PropertyValue<Restrictions<'a>, &'a M>;

	fn into_iter(self) -> Self::IntoIter {
		WithRestrictionsIter {
			value: Some(self.value),
		}
	}
}

pub struct WithRestrictionsIter<'a, M> {
	value: Option<Meta<Restrictions<'a>, &'a M>>,
}

impl<'a, M> Iterator for WithRestrictionsIter<'a, M> {
	type Item = PropertyValue<Restrictions<'a>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.value
			.take()
			.map(|value| PropertyValue::new(None, value))
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
		MetaOption<restriction::real::Restrictions, M>,
	),
	Float(
		Meta<TId<crate::ty::DataType<M>>, M>,
		MetaOption<restriction::float::Restrictions, M>,
	),
	Double(
		Meta<TId<crate::ty::DataType<M>>, M>,
		MetaOption<restriction::double::Restrictions, M>,
	),
	String(
		Meta<TId<crate::ty::DataType<M>>, M>,
		MetaOption<restriction::string::Restrictions, M>,
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

	pub fn with_restrictions(&self) -> Option<WithRestrictions<M>> {
		match self {
			Self::Real(_, r) => r
				.as_ref()
				.map(|r| WithRestrictions::new(r.borrow().map(Restrictions::Real))),
			Self::Float(_, r) => r
				.as_ref()
				.map(|r| WithRestrictions::new(r.borrow().map(Restrictions::Float))),
			Self::Double(_, r) => r
				.as_ref()
				.map(|r| WithRestrictions::new(r.borrow().map(Restrictions::Double))),
			Self::String(_, r) => r
				.as_ref()
				.map(|r| WithRestrictions::new(r.borrow().map(Restrictions::String))),
			_ => None,
		}
	}

	pub fn restrictions(&self) -> Option<Restrictions> {
		match self {
			Self::Real(_, r) => r.value().map(Restrictions::Real),
			Self::Float(_, r) => r.value().map(Restrictions::Float),
			Self::Double(_, r) => r.value().map(Restrictions::Double),
			Self::String(_, r) => r.value().map(Restrictions::String),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	OnDatatype(Option<TId<UnknownProperty>>),
	WithRestrictions(Option<TId<UnknownProperty>>),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::{Owl, Term};
		match self {
			Self::OnDatatype(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::OnDatatype))),
			Self::OnDatatype(Some(p)) => p.id(),
			Self::WithRestrictions(None) => {
				Id::Iri(IriIndex::Iri(Term::Owl(Owl::WithRestrictions)))
			}
			Self::WithRestrictions(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::{Owl, Term};
		match self {
			Self::OnDatatype(None) => Some(Term::Owl(Owl::OnDatatype)),
			Self::WithRestrictions(None) => Some(Term::Owl(Owl::WithRestrictions)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::OnDatatype(None) => PropertyName::Resource("restricted datatype"),
			Self::OnDatatype(Some(p)) => PropertyName::Other(*p),
			Self::WithRestrictions(None) => PropertyName::Resource("datatype restrictions"),
			Self::WithRestrictions(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_type(&self) -> bool {
		matches!(self, Self::OnDatatype(_))
	}

	pub fn expect_layout(&self) -> bool {
		false
	}
}

pub enum ClassBindingRef<'a, M> {
	OnDatatype(Option<TId<UnknownProperty>>, TId<crate::ty::DataType<M>>),
	WithRestrictions(Option<TId<UnknownProperty>>, Restrictions<'a>),
}

pub type BindingRef<'a, M> = ClassBindingRef<'a, M>;

impl<'a, M> ClassBindingRef<'a, M> {
	pub fn property(&self) -> Property {
		match self {
			Self::OnDatatype(p, _) => Property::OnDatatype(*p),
			Self::WithRestrictions(p, _) => Property::WithRestrictions(*p),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::OnDatatype(_, v) => BindingValueRef::DataType(*v),
			Self::WithRestrictions(_, v) => BindingValueRef::DatatypeRestrictions(*v),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	on_datatype: Option<OnDatatypeIter<'a, M>>,
	with_restrictions: Option<WithRestrictionsIter<'a, M>>,
}

pub type Bindings<'a, M> = ClassBindings<'a, M>;

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.on_datatype
			.as_mut()
			.and_then(OnDatatypeIter::next)
			.or_else(|| {
				self.with_restrictions.as_mut().and_then(|r| {
					r.next()
						.map(|m| m.into_class_binding(ClassBindingRef::WithRestrictions))
				})
			})
	}
}
