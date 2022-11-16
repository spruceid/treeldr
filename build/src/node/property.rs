use std::fmt;

use contextual::DisplayWithContext;
use locspan::Meta;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, Id, IriIndex};

use crate::{multiple, single};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum List {
	First,
	Rest
}

impl List {
	fn term(&self) -> treeldr::vocab::Term {
		use treeldr::vocab::{Term, Rdf};
		match self {
			Self::First => Term::Rdf(Rdf::First),
			Self::Rest => Term::Rdf(Rdf::Rest),
		}
	}

	fn name(&self) -> &'static str {
		match self {
			Self::First => "first item",
			Self::Rest => "rest"
		}
	}
}

pub enum Name {
	BuiltIn(&'static str),
	Other(Id),
}

impl<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>> DisplayWithContext<V> for Name {
	fn fmt_with(&self, context: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::BuiltIn(name) => fmt::Display::fmt(name, f),
			Self::Other(id) => id.fmt_with(context, f),
		}
	}
}

pub enum BindingRef<'a, M> {
	Resource(crate::resource::BindingRef<'a, M>),
	Class(crate::ty::BindingRef<'a, M>),
	DatatypeRestriction(crate::ty::datatype::restriction::BindingRef<'a, M>),
	Property(crate::prop::BindingRef<'a, M>),
	// Layout(crate::layout::BindingRef<'a, M>),
	// LayoutField(crate::layout::field::BindingRef<'a, M>),
	// LayoutVariant(crate::layout::variant::BindingRef<'a, M>),
	LayoutRestriction(crate::layout::restriction::BindingRef<'a, M>),
	List(crate::list::BindingRef<'a, M>)
}

/// Iterator over the bindings of a given node.
pub struct Bindings<'a, M> {
	resource: crate::resource::Bindings<'a, M>,
	class: crate::ty::Bindings<'a, M>,
	datatype_restriction: crate::ty::datatype::restriction::Bindings<'a, M>,
	property: crate::prop::Bindings<'a, M>,
	// layout: crate::layout::Bindings<'a, M>,
	// layout_field: crate::layout::field::Bindings<'a, M>,
	// layout_variant: crate::layout::variant::Bindings<'a, M>,
	layout_restriction: crate::layout::restriction::Bindings<'a, M>,
	list: crate::list::Bindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = BindingRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		todo!()
		// self.resource
		// 	.next()
		// 	.map(BindingRef::Resource)
		// 	.or_else(|| {
		// 		self.class
		// 			.next()
		// 			.map(BindingRef::Class)
		// 			.or_else(|| {
		// 				self.datatype_restriction
		// 					.next()
		// 					.map(BindingRef::DatatypeRestriction)
		// 					.or_else(|| {
		// 						self.property
		// 							.next()
		// 							.map(BindingRef::Property)
		// 							.or_else(|| {
		// 								self.layout
		// 									.next()
		// 									.map(BindingRef::Layout)
		// 									.or_else(|| {
		// 										self.layout_field
		// 											.next()
		// 											.map(BindingRef::LayoutField)
		// 											.or_else(|| {
		// 												self.layout_variant
		// 													.next()
		// 													.map(BindingRef::LayoutVariant)
		// 													.or_else(|| {
		// 														self.layout_restriction
		// 															.next()
		// 															.map(BindingRef::LayoutRestriction)
		// 															.or_else(|| {
		// 																self.list
		// 																	.next()
		// 																	.map(BindingRef::List)
		// 															})
		// 													})
		// 											})
		// 									})
		// 							})
		// 					})
		// 			})
		// 	})
	}
}